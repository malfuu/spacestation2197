use chumsky::extra;
use chumsky::prelude::*;

use super::{BlockCollection, RBlock, ROperation};

const OP_ADD: &str = "add";
const OP_SUB: &str = "sub";
const OP_MUL: &str = "mul";
const OP_DIV: &str = "div";
const OP_JUMP: &str = "jump";

#[derive(Debug, Clone)]
enum LineItem {
    Header(String),
    Op(ROperation),
}

pub(super) fn parse_reaction<'a>(text: &'a str) -> Result<BlockCollection, String> {
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

    let block_prefix = just::<_, _, MyExtra<'a>>(':').ignore_then(identation.clone());

    // giga train of repeated code
    let add_op = just::<_, _, MyExtra<'a>>(OP_ADD)
        .then_ignore(at_least_one_space.clone())
        .ignore_then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .map(|((dest, src1), src2)| ROperation::Add(dest, src1, src2));

    let sub_op = just::<_, _, MyExtra<'a>>(OP_SUB)
        .then_ignore(at_least_one_space.clone())
        .ignore_then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .map(|((dest, src1), src2)| ROperation::Sub(dest, src1, src2));

    let mul_op = just::<_, _, MyExtra<'a>>(OP_MUL)
        .then_ignore(at_least_one_space.clone())
        .ignore_then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .map(|((dest, src1), src2)| ROperation::Mul(dest, src1, src2));

    let div_op = just::<_, _, MyExtra<'a>>(OP_DIV)
        .then_ignore(at_least_one_space.clone())
        .ignore_then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .then_ignore(at_least_one_space.clone())
        .then(identation.clone())
        .map(|((dest, src1), src2)| ROperation::Div(dest, src1, src2));

    let jump_op = just::<_, _, MyExtra<'a>>(OP_JUMP)
        .then_ignore(at_least_one_space.clone())
        .ignore_then(identation.clone())
        .map(ROperation::Jump);

    // gigatrain of repeated code over -- thank you for visiting shitcode

    let operation = add_op
        .or(sub_op)
        .or(mul_op)
        .or(div_op)
        .or(jump_op);

    let line = block_prefix
        .map(LineItem::Header)
        .or(operation.map(LineItem::Op))
        .padded_by(horiz_space);

    let newline = just::<_, _, MyExtra<'a>>('\n');

    let parser = line
        .separated_by(newline.repeated().at_least(1))
        .collect::<Vec<_>>()
        .padded_by(newline.repeated())
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

    #[test]
    fn parse_error_no_block() {
        let input = "add a b c";
        let parsed = parse_reaction(input);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_invalid_syntax() {
        let input = ":start\nadd a b";
        let parsed = parse_reaction(input);
        assert!(parsed.is_err());
    }
}
