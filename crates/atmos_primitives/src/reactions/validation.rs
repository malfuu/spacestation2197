use petgraph::algo::toposort;
use petgraph::graph::DiGraph;

use std::collections::HashMap;

use super::BLOCK_END;
use super::builder::{ParsedReactionFunction, REndOperation};

/// Returns `true` if the reaction has no loops.
pub(super) fn is_reaction_flow_valid(parsed: &ParsedReactionFunction) -> bool {
    let mut graph = DiGraph::<&str, ()>::new();
    let mut nodes = HashMap::new();

    for block in &parsed.blocks {
        let idx = graph.add_node(block.name.as_str());
        nodes.insert(block.name.as_str(), idx);
    }

    for block in &parsed.blocks {
        let from_idx = *nodes.get(block.name.as_str()).unwrap();
        let targets = match &block.last {
            REndOperation::Jump(target) => vec![target.as_str()],
            REndOperation::Brif(_, true_block, false_block) => {
                vec![true_block.as_str(), false_block.as_str()]
            }
        };

        for target in targets {
            if target != BLOCK_END
                && let Some(&to_idx) = nodes.get(target)
            {
                graph.add_edge(from_idx, to_idx, ());
            }
        }
    }

    toposort(&graph, None).is_ok()
}
