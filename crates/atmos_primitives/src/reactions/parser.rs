use chumsky::extra;
use chumsky::prelude::*;
use cranelift::prelude::FloatCC;

use super::builder::{
    ParsedReactionFunction, RBlock, REndOperation, ROperation, VarDeclaration, VarType,
};
use crate::gas_list::GasList;

const BLOCK_PREFIX_CHARACTER: char = ':';
const COMMENT_CHARACTER: char = ';';
// const GASTYPE_CHARACTER: char = '#';

const KEYWORD_DECLARE: &str = "declare";
const TYPE_SCALAR: &str = "scalar";
const TYPE_VECTOR: &str = "vector";
const TYPE_BOOL: &str = "bool";

const OP_ADD: &str = "add";
const OP_SUB: &str = "sub";
const OP_MUL: &str = "mul";
const OP_DIV: &str = "div";
// const OP_MAX: &str = "max";
// const OP_MIN: &str = "min";
// const OP_ABS: &str = "abs";
const OP_CMP: &str = "cmp";

const OP_REACTED: &str = "reacted";
const OP_EXTRACT: &str = "extract";
const OP_INSERT: &str = "insert";

const OP_JUMP: &str = "jump";
const OP_BRIF: &str = "brif";

#[derive(Debug, Clone)]
enum LineItem {
    Header(String),
    Op(ROperation),
    EndOp(REndOperation),
    Declaration(String, VarType, Option<f32>),
}

pub(super) fn parse_reaction<'a>(
    text: &'a str,
    gas_list: &GasList,
) -> Result<ParsedReactionFunction, String> {
    type MyExtra<'a> = extra::Err<Rich<'a, char>>;

    let horiz_space = any::<_, MyExtra<'a>>()
        .filter(|c: &char| *c == ' ')
        .repeated()
        .ignored();

    let at_least_one_space = any::<_, MyExtra<'a>>()
        .filter(|c: &char| *c == ' ')
        .repeated()
        .at_least(1)
        .ignored();

    let identation = any::<_, MyExtra<'a>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_')
        .repeated()
        .at_least(1)
        .collect::<String>();

    let block_prefix = just::<_, _, MyExtra<'a>>(BLOCK_PREFIX_CHARACTER)
        .ignore_then(identation)
        .or(identation.then_ignore(just::<_, _, MyExtra<'a>>(BLOCK_PREFIX_CHARACTER)));

    // giga train of repeated code
    let add_op = just::<_, _, MyExtra<'a>>(OP_ADD)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .map(|((dest, src1), src2)| ROperation::Add(dest, src1, src2));

    let sub_op = just::<_, _, MyExtra<'a>>(OP_SUB)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .map(|((dest, src1), src2)| ROperation::Sub(dest, src1, src2));

    let mul_op = just::<_, _, MyExtra<'a>>(OP_MUL)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .map(|((dest, src1), src2)| ROperation::Mul(dest, src1, src2));

    let div_op = just::<_, _, MyExtra<'a>>(OP_DIV)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .map(|((dest, src1), src2)| ROperation::Div(dest, src1, src2));

    let jump_op = just::<_, _, MyExtra<'a>>(OP_JUMP)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .map(REndOperation::Jump);

    let brif_op = just::<_, _, MyExtra<'a>>(OP_BRIF)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .map(|((cond, true_block), false_block)| {
            REndOperation::Brif(cond, true_block, false_block)
        });

    let reacted_op = just::<_, _, MyExtra<'a>>(OP_REACTED).map(|_| ROperation::Reacted);

    let gas_id_parser = identation.try_map(|name, span| {
        gas_list
            .try_get_gas_id_by_name(&name)
            .ok_or_else(|| Rich::custom(span, format!("Invalid gas name: {}", name)))
    });

    let extract_op = just::<_, _, MyExtra<'a>>(OP_EXTRACT)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .then_ignore(at_least_one_space)
        .then(gas_id_parser)
        .map(|((dest, src), gas_id)| ROperation::Extract(dest, src, gas_id));

    let insert_op = just::<_, _, MyExtra<'a>>(OP_INSERT)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(gas_id_parser)
        .then_ignore(at_least_one_space)
        .then(identation)
        .map(|((dest, gas_id), src)| ROperation::Insert(dest, gas_id, src));

    let cond_parser = just::<_, _, MyExtra<'a>>("eq")
        .to(FloatCC::Equal)
        .or(just::<_, _, MyExtra<'a>>("ne").to(FloatCC::NotEqual))
        .or(just::<_, _, MyExtra<'a>>("lt").to(FloatCC::LessThan))
        .or(just::<_, _, MyExtra<'a>>("le").to(FloatCC::LessThanOrEqual))
        .or(just::<_, _, MyExtra<'a>>("gt").to(FloatCC::GreaterThan))
        .or(just::<_, _, MyExtra<'a>>("ge").to(FloatCC::GreaterThanOrEqual));

    let cmp_op = just::<_, _, MyExtra<'a>>(OP_CMP)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(cond_parser)
        .then_ignore(at_least_one_space)
        .then(identation)
        .then_ignore(at_least_one_space)
        .then(identation)
        .map(|(((dest, cond), src1), src2)| ROperation::Cmp(dest, cond, src1, src2));

    // gigatrain of repeated code over -- thank you for visiting shitcode

    let operation = add_op
        .or(sub_op)
        .or(mul_op)
        .or(div_op)
        .or(cmp_op)
        .or(reacted_op)
        .or(extract_op)
        .or(insert_op);

    let end_operation = jump_op
        .map(LineItem::EndOp)
        .or(brif_op.map(LineItem::EndOp));

    let comment = just::<_, _, MyExtra<'a>>(COMMENT_CHARACTER)
        .then(any().filter(|c: &char| *c != '\n').repeated())
        .ignored();

    let spacing = horiz_space.then(comment.or_not()).ignored();

    let digits = any::<_, MyExtra<'a>>()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(1)
        .collect::<String>();

    let float_literal = just('-')
        .or_not()
        .then(digits)
        .then(just('.').ignore_then(digits).or_not())
        .try_map(|((sign, int_part), frac_part), span| {
            let s = match frac_part {
                Some(frac) => format!("{}.{}", int_part, frac),
                None => int_part,
            };
            let mut val = s
                .parse::<f32>()
                .map_err(|e| Rich::custom(span, format!("Invalid float: {}", e)))?;

            if sign.is_some() {
                val = -val;
            }

            Ok(val)
        });

    let decl_scalar = just::<_, _, MyExtra<'a>>(TYPE_SCALAR)
        .then(at_least_one_space.ignore_then(float_literal).or_not())
        .map(|(_, val)| (VarType::Scalar, val));

    let decl_vector = just::<_, _, MyExtra<'a>>(TYPE_VECTOR)
        .then(at_least_one_space.ignore_then(float_literal).or_not())
        .map(|(_, val)| (VarType::Vector, val));

    let decl_bool = just::<_, _, MyExtra<'a>>(TYPE_BOOL).map(|_| (VarType::Bool, None));

    let declaration = just::<_, _, MyExtra<'a>>(KEYWORD_DECLARE)
        .then_ignore(at_least_one_space)
        .ignore_then(identation)
        .then_ignore(at_least_one_space)
        .then(decl_scalar.or(decl_vector).or(decl_bool))
        .map(|(name, (ty, init))| LineItem::Declaration(name, ty, init));

    let line_item = block_prefix
        .map(LineItem::Header)
        .or(end_operation)
        .or(operation.map(LineItem::Op))
        .or(declaration)
        .padded_by(spacing);

    let newline = just::<_, _, MyExtra<'a>>('\n');

    let separator = spacing
        .then(newline)
        .then(spacing.or_not())
        .repeated()
        .at_least(1)
        .ignored();

    let parser = line_item
        .separated_by(separator)
        .collect::<Vec<_>>()
        .padded_by(separator.or_not())
        .then_ignore(end::<_, MyExtra<'a>>());

    let lines = parser.parse(text).into_result().map_err(|errs| {
        errs.into_iter()
            .map(|e: Rich<'a, char>| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    // transform lines into blocks

    /// exists because I couldnt figure out a better way cuz of last operation
    struct ProtoBlock {
        name: String,
        operations: Vec<ROperation>,
        last: Option<REndOperation>,
    }

    let mut proto_blocks = Vec::new();
    let mut current_block: Option<ProtoBlock> = None;
    let mut declarations = Vec::new();

    if lines.is_empty() {
        return Err("no lines in reaction!".into());
    }

    for line in lines {
        match line {
            LineItem::Header(name) => {
                if let Some(block) = current_block.take() {
                    proto_blocks.push(block);
                }
                current_block = Some(ProtoBlock {
                    name,
                    operations: Vec::new(),
                    last: None,
                });
            }
            LineItem::Op(op) => {
                if let Some(block) = current_block.as_mut() {
                    assert!(
                        block.last.is_none(),
                        "operation declared after block terminator"
                    );
                    block.operations.push(op);
                } else {
                    return Err("op declared out of block".into());
                }
            }
            LineItem::EndOp(end_op) => {
                if let Some(block) = current_block.as_mut() {
                    assert!(block.last.is_none(), "multiple block terminators declared");
                    block.last = Some(end_op);
                } else {
                    return Err("end op declared out of block".into());
                }
            }
            LineItem::Declaration(name, ty, init) => {
                declarations.push(VarDeclaration { name, ty, init });
            }
        }
    }

    if let Some(block) = current_block {
        proto_blocks.push(block);
    }

    let mut blocks = Vec::new();
    for pb in proto_blocks {
        let last = pb.last.expect("Block missing jump or brif");
        blocks.push(RBlock {
            name: pb.name,
            operations: pb.operations,
            last,
        });
    }

    Ok(ParsedReactionFunction {
        declarations,
        blocks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Gas;

    fn make_test_gas_list() -> GasList {
        GasList::new(vec![
            Gas {
                gas_id: 0,
                name: "gas_a".to_string(),
                molar_heat_capacity: 21.1,
            },
            Gas {
                gas_id: 1,
                name: "gas_b".to_string(),
                molar_heat_capacity: 20.7,
            },
        ])
    }

    #[test]
    fn parse_error_no_block() {
        let input = "add a b c";
        let parsed = parse_reaction(input, &make_test_gas_list());
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_invalid_syntax() {
        let input = ":start\nadd a b";
        let parsed = parse_reaction(input, &make_test_gas_list());
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_comments() {
        let input = "
            ; comment
            :start ; block comment
            reacted ; op comment

            ; another comment line
            jump end
        ";
        let parsed = parse_reaction(input, &make_test_gas_list()).unwrap();
        assert_eq!(parsed.blocks.len(), 1);
        assert_eq!(parsed.blocks[0].name, "start");
        assert_eq!(parsed.blocks[0].operations.len(), 1);
        assert_eq!(parsed.blocks[0].operations[0], ROperation::Reacted);
        assert_eq!(
            parsed.blocks[0].last,
            REndOperation::Jump("end".to_string())
        );
    }
}
