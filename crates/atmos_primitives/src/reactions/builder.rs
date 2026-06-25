use std::{collections::HashMap, error::Error, sync::Arc};

use cranelift::{
    codegen::{
        ir::types::{F32, F32X4},
        isa::TargetIsa,
    },
    prelude::*,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::{
    GasId,
    reactions::{
        BLOCK_END, BLOCK_START, ReactionFn, ReactionResult, VARIABLE_ENERGY,
        VARIABLE_HEAT_CAPACITY, VARIABLE_MOLES, VARIABLE_PRESSURE, VARIABLE_TEMPERATURE,
        VARIABLE_TOTAL_MOLES,
    },
};

const CRANELIFT_OPT_LEVEL: &str = "speed";
const CRANELIFT_OPT_PIC: &str = "false";

pub(super) type BlockName = String;
pub(super) type VarName = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VarType {
    Scalar, // f32
    Vector, // f32x16
    Bool,   // i8
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ROperation {
    /// Adds two values. Scalar and Vector.
    Add(VarName, VarName, VarName),
    /// Subtracts two values. Scalar and Vector.
    Sub(VarName, VarName, VarName),
    /// Multiplies two values. Scalar and Vector.
    Mul(VarName, VarName, VarName),
    /// Divides two values. Scalar and Vector.
    Div(VarName, VarName, VarName),
    /// Returns the maximum value for each value. Scalar and Vector.
    Max(VarName, VarName, VarName),
    /// Returns the minimum value for each value. Scalar and Vector.
    Min(VarName, VarName, VarName),
    /// Calculates absolute of a value. Scalar and Vector.
    Abs(VarName, VarName),
    /// Compares two scalar values and return a boolean. No Vectors.
    Cmp(VarName, FloatCC, VarName, VarName),
    /// Marks that the reaction has reacted. Set only.
    Reacted,
    /// Extracts a scalar value from a vector at `GasId`.
    Extract(VarName, VarName, GasId),
    /// Inserts a scalar value into a vector.
    Insert(VarName, GasId, VarName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum REndOperation {
    /// Jumps to another block unconditionally.
    Jump(BlockName),
    /// Jumps to the first block if condition is `true`,
    /// otherwise to the second block
    Brif(VarName, BlockName, BlockName),
}

type VectorVariable = [Variable; 4]; // sub-vectors

#[allow(dead_code)]
enum CraneliftVariable {
    Scalar(Variable),
    Vector(VectorVariable),
    Bool(Variable),
}

/// short hand helper function [`CraneliftVariable::Scalar`]
fn cl_scalar(var: Variable) -> CraneliftVariable {
    CraneliftVariable::Scalar(var)
}

/// short hand helper function [`CraneliftVariable::Vector`]
fn cl_vector(variable: VectorVariable) -> CraneliftVariable {
    CraneliftVariable::Vector(variable)
}

type VariablesMap = HashMap<VarName, CraneliftVariable>;
type BlocksMap = HashMap<BlockName, Block>;

#[derive(Debug, Clone)]
pub(super) struct RBlock {
    pub name: String,
    pub operations: Vec<ROperation>,
    pub last: REndOperation,
}

#[derive(Debug, Clone)]
pub(super) struct VarDeclaration {
    pub name: VarName,
    pub ty: VarType,
}

#[derive(Debug, Clone)]
pub(super) struct ParsedReactionFunction {
    pub declarations: Vec<VarDeclaration>,
    pub blocks: Vec<RBlock>,
}

fn get_reaction_settings_builder() -> settings::Builder {
    let mut builder = settings::builder();

    builder.set("opt_level", CRANELIFT_OPT_LEVEL).unwrap();
    builder.set("is_pic", CRANELIFT_OPT_PIC).unwrap();

    builder
}

fn get_isa(flags: settings::Flags) -> Result<Arc<dyn TargetIsa>, Box<dyn Error>> {
    let isa_builder = cranelift_native::builder()?;

    let isa = isa_builder.finish(flags)?;

    Ok(isa)
}

fn get_signature(module: &mut JITModule) -> Signature {
    let mut signature = module.make_signature();

    signature.params.push(AbiParam::new(types::I64)); // &mut f32x16 moles
    signature.params.push(AbiParam::new(types::I64)); // &mut f32 energy

    signature.params.push(AbiParam::new(types::F32)); // total_moles
    signature.params.push(AbiParam::new(types::F32)); // heat_capacity
    signature.params.push(AbiParam::new(types::F32)); // temperature
    signature.params.push(AbiParam::new(types::F32)); // pressure

    signature.returns.push(AbiParam::new(types::I32)); // ReactionResult
    signature
}

fn declare_scalar(builder: &mut FunctionBuilder) -> Variable {
    builder.declare_var(F32)
}

fn declare_vector(builder: &mut FunctionBuilder) -> [Variable; 4] {
    [
        builder.declare_var(F32X4),
        builder.declare_var(F32X4),
        builder.declare_var(F32X4),
        builder.declare_var(F32X4),
    ]
}

fn declare_blocks(builder: &mut FunctionBuilder, parsed: &ParsedReactionFunction) -> BlocksMap {
    let mut blocks_map = BlocksMap::new();

    for rblock in &parsed.blocks {
        let cl_block = builder.create_block();
        blocks_map.insert(rblock.name.clone(), cl_block);
    }

    let end_block = builder.create_block();
    blocks_map.insert("end".to_string(), end_block);

    blocks_map
}

struct BuiltinVariables {
    moles_ptr_var: Variable,
    energy_ptr_var: Variable,

    moles_var: VectorVariable,
    energy_var: Variable,

    total_moles_var: Variable,
    heat_capacity_var: Variable,
    temperature_var: Variable,
    pressure_var: Variable,

    reacted_var: Variable,
}

fn declare_builtin_variables(
    builder: &mut FunctionBuilder,
    variables: &mut VariablesMap,
) -> BuiltinVariables {
    let moles_var = declare_vector(builder);
    let energy_var = declare_scalar(builder);
    let total_moles_var = declare_scalar(builder);
    let heat_capacity_var = declare_scalar(builder);
    let temperature_var = declare_scalar(builder);
    let pressure_var = declare_scalar(builder);

    let moles_ptr_var = builder.declare_var(types::I64);
    let energy_ptr_var = builder.declare_var(types::I64);

    let reacted_var = builder.declare_var(types::I32);

    variables.insert(VARIABLE_MOLES.into(), cl_vector(moles_var));
    variables.insert(VARIABLE_ENERGY.into(), cl_scalar(energy_var));

    variables.insert(VARIABLE_TOTAL_MOLES.into(), cl_scalar(total_moles_var));
    variables.insert(VARIABLE_HEAT_CAPACITY.into(), cl_scalar(heat_capacity_var));
    variables.insert(VARIABLE_TEMPERATURE.into(), cl_scalar(temperature_var));
    variables.insert(VARIABLE_PRESSURE.into(), cl_scalar(pressure_var));

    BuiltinVariables {
        moles_var,
        energy_var,
        total_moles_var,
        heat_capacity_var,
        temperature_var,
        pressure_var,
        moles_ptr_var,
        energy_ptr_var,
        reacted_var,
    }
}

fn declare_user_variables(
    builder: &mut FunctionBuilder,
    declarations: &[VarDeclaration],
    variables: &mut VariablesMap,
) -> Result<(), Box<dyn Error>> {
    for declaration in declarations {
        if variables.contains_key(&declaration.name) {
            return Err(format!(
                "Cannot declare variable '{}' it is already built in.",
                declaration.name
            )
            .into());
        }

        match declaration.ty {
            VarType::Scalar => {
                let var = declare_scalar(builder);
                variables.insert(declaration.name.clone(), cl_scalar(var));
            }
            VarType::Vector => {
                let var = declare_vector(builder);
                variables.insert(declaration.name.clone(), cl_vector(var));
            }
            VarType::Bool => {
                let var = builder.declare_var(types::I8);
                variables.insert(declaration.name.clone(), CraneliftVariable::Bool(var));
            }
        }
    }
    Ok(())
}

fn build_start_block(
    builder: &mut FunctionBuilder,
    entry_block: Block,
    vars: &BuiltinVariables,
    first_block: Block,
) {
    builder.switch_to_block(entry_block);
    builder.append_block_params_for_function_params(entry_block);

    let moles_ptr = builder.block_params(entry_block)[0]; // &mut f32x16
    let energy_ptr = builder.block_params(entry_block)[1]; // &mut f32
    let total_moles = builder.block_params(entry_block)[2]; // f32
    let heat_capacity = builder.block_params(entry_block)[3]; // f32
    let temperature = builder.block_params(entry_block)[4]; // f32
    let pressure = builder.block_params(entry_block)[5]; // f32

    let flags = MemFlags::new();

    // definitions
    let m0 = builder.ins().load(types::F32X4, flags, moles_ptr, 0);
    let m1 = builder.ins().load(types::F32X4, flags, moles_ptr, 16);
    let m2 = builder.ins().load(types::F32X4, flags, moles_ptr, 32);
    let m3 = builder.ins().load(types::F32X4, flags, moles_ptr, 48);

    builder.def_var(vars.moles_var[0], m0);
    builder.def_var(vars.moles_var[1], m1);
    builder.def_var(vars.moles_var[2], m2);
    builder.def_var(vars.moles_var[3], m3);

    let energy_value = builder.ins().load(types::F32, flags, energy_ptr, 0);
    builder.def_var(vars.energy_var, energy_value);

    builder.def_var(vars.total_moles_var, total_moles);
    builder.def_var(vars.heat_capacity_var, heat_capacity);
    builder.def_var(vars.temperature_var, temperature);
    builder.def_var(vars.pressure_var, pressure);

    builder.def_var(vars.moles_ptr_var, moles_ptr);
    builder.def_var(vars.energy_ptr_var, energy_ptr);

    let zero_i32 = builder
        .ins()
        .iconst(types::I32, ReactionResult::DidNotReact as i64);
    builder.def_var(vars.reacted_var, zero_i32);

    builder.ins().jump(first_block, &[]);
}

fn build_end_block(builder: &mut FunctionBuilder, end_block: Block, vars: &BuiltinVariables) {
    builder.switch_to_block(end_block);

    let moles_ptr = builder.use_var(vars.moles_ptr_var);
    let energy_ptr = builder.use_var(vars.energy_ptr_var);

    let m0 = builder.use_var(vars.moles_var[0]);
    let m1 = builder.use_var(vars.moles_var[1]);
    let m2 = builder.use_var(vars.moles_var[2]);
    let m3 = builder.use_var(vars.moles_var[3]);

    let energy_value = builder.use_var(vars.energy_var);

    let flags = MemFlags::new();

    builder.ins().store(flags, m0, moles_ptr, 0);
    builder.ins().store(flags, m1, moles_ptr, 16);
    builder.ins().store(flags, m2, moles_ptr, 32);
    builder.ins().store(flags, m3, moles_ptr, 48);

    builder.ins().store(flags, energy_value, energy_ptr, 0);

    let ret_val = builder.use_var(vars.reacted_var);
    builder.ins().return_(&[ret_val]);
}

fn build_blocks(
    builder: &mut FunctionBuilder,
    parsed: &ParsedReactionFunction,
    blocks: &BlocksMap,
    variables: &mut VariablesMap,
    reacted_var: Variable,
) {
    for rblock in &parsed.blocks {
        let block = blocks.get(&rblock.name).expect("block should exist.");
        builder.switch_to_block(*block);

        for operation in &rblock.operations {
            match operation {
                ROperation::Cmp(dest, cond, src1, src2) => {
                    let cond_var = match variables.get(dest).expect("dest not found") {
                        CraneliftVariable::Bool(v) => *v,
                        _ => panic!(),
                    };
                    let v1 = match variables.get(src1).expect("src1 not found") {
                        CraneliftVariable::Scalar(s) => builder.use_var(*s),
                        _ => panic!(),
                    };
                    let v2 = match variables.get(src2).expect("src2 not found") {
                        CraneliftVariable::Scalar(s) => builder.use_var(*s),
                        _ => panic!(),
                    };
                    let res = builder.ins().fcmp(*cond, v1, v2);
                    builder.def_var(cond_var, res);
                }
                ROperation::Reacted => {
                    let one_i32 = builder
                        .ins()
                        .iconst(types::I32, ReactionResult::Reacted as i64);
                    builder.def_var(reacted_var, one_i32);
                }
                _ => {}
            }
        }

        match &rblock.last {
            REndOperation::Jump(target) => {
                let target_block = blocks.get(target).expect("target block should exist.");
                builder.ins().jump(*target_block, &[]);
            }
            REndOperation::Brif(cond_name, true_block_name, false_block_name) => {
                let cond_var = match variables
                    .get(cond_name)
                    .expect("condition variable not found")
                {
                    CraneliftVariable::Bool(v) => *v,
                    _ => panic!("expected boolean variable for brif condition"),
                };
                let cond_val = builder.use_var(cond_var);

                let true_block = blocks.get(true_block_name).expect("true block not found");
                let false_block = blocks.get(false_block_name).expect("false block not found");

                builder
                    .ins()
                    .brif(cond_val, *true_block, &[], *false_block, &[]);
            }
        }
    }
}

fn build_function(
    module: &mut JITModule,
    func_id: cranelift_module::FuncId,
    signature: Signature,
    parsed: ParsedReactionFunction,
) -> Result<(), Box<dyn Error>> {
    let mut ctx = module.make_context();
    ctx.func.signature = signature;

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

    let entry_block = builder.create_block();
    let blocks = declare_blocks(&mut builder, &parsed);

    let mut variables = VariablesMap::new();
    let vars = declare_builtin_variables(&mut builder, &mut variables);

    declare_user_variables(&mut builder, &parsed.declarations, &mut variables)?;

    let start_block = *blocks.get(BLOCK_START).expect("start block not found");

    build_start_block(&mut builder, entry_block, &vars, start_block);

    build_blocks(
        &mut builder,
        &parsed,
        &blocks,
        &mut variables,
        vars.reacted_var,
    );

    let end_block = *blocks.get(BLOCK_END).unwrap();
    build_end_block(&mut builder, end_block, &vars);

    builder.seal_all_blocks();
    builder.finalize();

    module.define_function(func_id, &mut ctx)?;
    module.clear_context(&mut ctx);

    Ok(())
}

pub(super) fn build_reactions(
    parsed_reactions: Vec<ParsedReactionFunction>,
) -> Result<(JITModule, Vec<ReactionFn>), Box<dyn Error>> {
    let settings_builder = get_reaction_settings_builder();
    let flags = settings::Flags::new(settings_builder);

    let isa = get_isa(flags)?;

    if isa.pointer_type() != types::I64 {
        return Err("64 bit pointers required.".into());
    }

    let jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    let mut module = JITModule::new(jit_builder);

    let signature = get_signature(&mut module);

    let mut reactions = Vec::with_capacity(parsed_reactions.len());
    let mut function_ids = Vec::with_capacity(parsed_reactions.len());

    for (idx, blocks) in parsed_reactions.into_iter().enumerate() {
        let name = format!("reaction_{}", idx);
        let function_id = module.declare_function(&name, Linkage::Export, &signature)?;
        function_ids.push(function_id);

        build_function(&mut module, function_id, signature.clone(), blocks)?;
    }

    module.finalize_definitions()?;

    for function_id in function_ids {
        let ptr = module.get_finalized_function(function_id);
        let reaction_fn = unsafe { std::mem::transmute::<*const u8, ReactionFn>(ptr) };
        reactions.push(reaction_fn);
    }

    Ok((module, reactions))
}

