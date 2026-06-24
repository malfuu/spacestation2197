use chumsky::extra;
use chumsky::prelude::*;

use super::builder::{BlockCollection, RBlock, ROperation};
use crate::gas_list::GasList;

const BLOCK_PREFIX_CHARACTER: char = ':';
const COMMENT_CHARACTER: char = ';';
// const GASTYPE_CHARACTER: char = '#';

const OP_ADD: &str = "add";
const OP_SUB: &str = "sub";
const OP_MUL: &str = "mul";
const OP_DIV: &str = "div";
const OP_JUMP: &str = "jump";
const OP_REACTED: &str = "reacted";

#[derive(Debug, Clone)]
enum LineItem {
    Header(String),
    Op(ROperation),
}

pub(super) fn parse_reaction<'a>(
    text: &'a str,
    _gas_list: &GasList,
) -> Result<BlockCollection, String> {
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

    let block_prefix = just::<_, _, MyExtra<'a>>(BLOCK_PREFIX_CHARACTER).ignore_then(identation);

    // TODO: enable
    // let gas_id_ref = just::<_, _, MyExtra<'a>>(GASTYPE_CHARACTER)
    //     .ignore_then(identation.clone())
    //     .try_map(|name, span| {
    //         gas_list
    //             .try_get_gas_id_by_name(&name)
    //             .map(|id| id.to_string())
    //             .ok_or_else(|| Rich::custom(span, format!("Invalid gas name: {}", name)))
    //     });

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
        .map(ROperation::Jump);

    let reacted_op = just::<_, _, MyExtra<'a>>(OP_REACTED).map(|_| ROperation::Reacted);

    // gigatrain of repeated code over -- thank you for visiting shitcode

    let operation = add_op
        .or(sub_op)
        .or(mul_op)
        .or(div_op)
        .or(jump_op)
        .or(reacted_op);

    let comment = just::<_, _, MyExtra<'a>>(COMMENT_CHARACTER)
        .then(any().filter(|c: &char| *c != '\n').repeated())
        .ignored();

    let spacing = horiz_space.then(comment.or_not()).ignored();

    let line_item = block_prefix
        .map(LineItem::Header)
        .or(operation.map(LineItem::Op))
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
    let mut blocks = Vec::new();
    let mut current_block: Option<RBlock> = None;

    if lines.is_empty() {
        return Err("no lines in reaction!".into());
    }

    for line in lines {
        match line {
            LineItem::Header(name) => {
                if let Some(block) = current_block.take() {
                    blocks.push(block);
                }
                current_block = Some(RBlock {
                    name,
                    operations: Vec::new(),
                });
            }
            LineItem::Op(op) => {
                if let Some(block) = current_block.as_mut() {
                    block.operations.push(op);
                } else {
                    return Err("op declared out of instruction".into());
                }
            }
        }
    }

    if let Some(block) = current_block {
        blocks.push(block);
    }

    Ok(blocks)
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
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "start");
        assert_eq!(parsed[0].operations.len(), 2);
        assert_eq!(parsed[0].operations[0], ROperation::Reacted);
        assert_eq!(parsed[0].operations[1], ROperation::Jump("end".to_string()));
    }
}
