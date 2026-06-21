use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc,
};

use cranelift::{codegen::isa::TargetIsa, prelude::*};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::reactions::{BlockCollection, ROperation, ReactionFn};

const CRANELIFT_OPT_LEVEL: &'static str = "speed";

fn get_reaction_settings_builder() -> settings::Builder {
    let mut builder = settings::builder();

    builder.set("opt_level", CRANELIFT_OPT_LEVEL).unwrap();

    builder
}

fn get_isa(flags: settings::Flags) -> Result<Arc<dyn TargetIsa>, Box<dyn Error>> {
    let isa_builder = cranelift_native::builder()?;

    let isa = isa_builder.finish(flags)?;

    Ok(isa)
}

fn get_signature(module: &mut JITModule) -> Signature {
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(types::I64)); // &mut f32x16
    sig.returns.push(AbiParam::new(types::I32)); // ReactionResult
    sig
}

/// transform
fn start_block(
    builder: &mut FunctionBuilder,
    variables: &HashMap<String, Variable>,
    first_user_block: Option<Block>,
    end_block: Block,
) -> (Block, Value) {
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);

    let ptr_val = builder.block_params(entry_block)[0];

    let flags = MemFlags::new();
    let v0_val = builder.ins().load(types::F32X4, flags, ptr_val, 0);
    let v1_val = builder.ins().load(types::F32X4, flags, ptr_val, 16);
    let v2_val = builder.ins().load(types::F32X4, flags, ptr_val, 32);
    let v3_val = builder.ins().load(types::F32X4, flags, ptr_val, 48);

    builder.def_var(*variables.get("v0").unwrap(), v0_val);
    builder.def_var(*variables.get("v1").unwrap(), v1_val);
    builder.def_var(*variables.get("v2").unwrap(), v2_val);
    builder.def_var(*variables.get("v3").unwrap(), v3_val);

    // init user variables to a zero vector f32x4.
    let zero = builder.ins().f32const(0.0);
    let zero_v = builder.ins().splat(types::F32X4, zero);
    for (name, var) in variables {
        if name != "v0" && name != "v1" && name != "v2" && name != "v3" {
            builder.def_var(*var, zero_v);
        }
    }

    if let Some(target) = first_user_block {
        builder.ins().jump(target, &[]);
    } else {
        builder.ins().jump(end_block, &[]);
    }

    (entry_block, ptr_val)
}

/// transforms
fn end_block(
    builder: &mut FunctionBuilder,
    variables: &HashMap<String, Variable>,
    ptr_val: Value,
    end_block: Block,
) {
    builder.switch_to_block(end_block);
    let v0_val = builder.use_var(*variables.get("v0").unwrap());
    let v1_val = builder.use_var(*variables.get("v1").unwrap());
    let v2_val = builder.use_var(*variables.get("v2").unwrap());
    let v3_val = builder.use_var(*variables.get("v3").unwrap());

    let flags = MemFlags::new();
    builder.ins().store(flags, v0_val, ptr_val, 0);
    builder.ins().store(flags, v1_val, ptr_val, 16);
    builder.ins().store(flags, v2_val, ptr_val, 32);
    builder.ins().store(flags, v3_val, ptr_val, 48);

    // Return ReactionResult::Reacted (1).
    let ret_val = builder.ins().iconst(types::I32, 1);
    builder.ins().return_(&[ret_val]);
}

fn build_function(
    module: &mut JITModule,
    func_id: cranelift_module::FuncId,
    sig: Signature,
    rblocks: BlockCollection,
) -> Result<(), Box<dyn Error>> {
    let mut ctx = module.make_context();
    ctx.func.signature = sig;

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

    // cl block per rblock
    let mut cranelift_blocks = HashMap::new();
    for rblock in &rblocks {
        let cl_block = builder.create_block();
        cranelift_blocks.insert(rblock.name.clone(), cl_block);
    }

    // end block
    let cl_end_block = builder.create_block();
    cranelift_blocks.insert("end".to_string(), cl_end_block);

    // we store our variables here
    let mut user_variable_names = HashSet::<String>::new();

    // moles variables
    user_variable_names.insert("v0".to_string());
    user_variable_names.insert("v1".to_string());
    user_variable_names.insert("v2".to_string());
    user_variable_names.insert("v3".to_string());

    for block in &rblocks {
        for op in &block.operations {
            match op {
                ROperation::Add(dest, src1, src2)
                | ROperation::Sub(dest, src1, src2)
                | ROperation::Mul(dest, src1, src2)
                | ROperation::Div(dest, src1, src2) => {
                    user_variable_names.insert(dest.clone());
                    user_variable_names.insert(src1.clone());
                    user_variable_names.insert(src2.clone());
                }
                ROperation::Jump(_) => {}
            }
        }
    }

    // we will use cranelift variables here
    let mut variables = HashMap::new();
    for variable_name in user_variable_names {
        let var = builder.declare_var(types::F32X4);
        variables.insert(variable_name, var);
    }

    let first_user_block = rblocks
        .first()
        .map(|fb| *cranelift_blocks.get(&fb.name).unwrap());

    let (_entry_block, ptr_val) =
        start_block(&mut builder, &variables, first_user_block, cl_end_block);

    // actual conversion of blocks
    for (_idx, block) in rblocks.iter().enumerate() {
        let cl_block = *cranelift_blocks.get(&block.name).unwrap();
        builder.switch_to_block(cl_block);

        for op in &block.operations {
            match op {
                ROperation::Add(dest, src1, src2) => {
                    let v1 = builder.use_var(*variables.get(src1).unwrap());
                    let v2 = builder.use_var(*variables.get(src2).unwrap());
                    let res = builder.ins().fadd(v1, v2);
                    builder.def_var(*variables.get(dest).unwrap(), res);
                }
                ROperation::Sub(dest, src1, src2) => {
                    let v1 = builder.use_var(*variables.get(src1).unwrap());
                    let v2 = builder.use_var(*variables.get(src2).unwrap());
                    let res = builder.ins().fsub(v1, v2);
                    builder.def_var(*variables.get(dest).unwrap(), res);
                }
                ROperation::Mul(dest, src1, src2) => {
                    let v1 = builder.use_var(*variables.get(src1).unwrap());
                    let v2 = builder.use_var(*variables.get(src2).unwrap());
                    let res = builder.ins().fmul(v1, v2);
                    builder.def_var(*variables.get(dest).unwrap(), res);
                }
                ROperation::Div(dest, src1, src2) => {
                    let v1 = builder.use_var(*variables.get(src1).unwrap());
                    let v2 = builder.use_var(*variables.get(src2).unwrap());
                    let res = builder.ins().fdiv(v1, v2);
                    builder.def_var(*variables.get(dest).unwrap(), res);
                }
                ROperation::Jump(target) => {
                    let target_block = *cranelift_blocks.get(target).unwrap();
                    builder.ins().jump(target_block, &[]);
                }
            }
        }
    }

    end_block(&mut builder, &variables, ptr_val, cl_end_block);

    builder.seal_all_blocks();
    builder.finalize();

    module.define_function(func_id, &mut ctx)?;
    module.clear_context(&mut ctx);

    Ok(())
}

pub(super) fn build_reactions(
    parsed_reactions: Vec<BlockCollection>,
) -> Result<(JITModule, Vec<ReactionFn>), Box<dyn Error>> {
    let settings_builder = get_reaction_settings_builder();
    let flags = settings::Flags::new(settings_builder);

    let isa = get_isa(flags)?;

    if isa.pointer_type() != types::I64 {
        return Err("64 bit pointers required.".into());
    }

    let jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    let mut module = JITModule::new(jit_builder);

    let sig = get_signature(&mut module);

    let mut reactions: Vec<ReactionFn> = vec![];
    let mut func_ids = Vec::new();

    for (idx, blocks) in parsed_reactions.into_iter().enumerate() {
        let name = format!("reaction_{}", idx);
        let func_id = module.declare_function(&name, Linkage::Export, &sig)?;
        func_ids.push(func_id);

        build_function(&mut module, func_id, sig.clone(), blocks)?;
    }

    module.finalize_definitions()?;

    for func_id in func_ids {
        let ptr = module.get_finalized_function(func_id);
        let reaction_fn = unsafe { std::mem::transmute::<*const u8, ReactionFn>(ptr) };
        reactions.push(reaction_fn);
    }

    Ok((module, reactions))
}

