//! Reactions of mixtures that do not necessarily follow the laws of physics.
mod builder;
pub(crate) mod parser;

use std::error::Error;

use bevy::prelude::*;

use cranelift_jit::JITModule;

use wide::f32x16;

use crate::reactions::{builder::build_reactions, parser::parse_reaction};

/// Reaction Fn applied to mixture values.
/// Filled with unsafe black magic due to its JIT compiled nature.
/// # SAFETY
/// Function lifetime associated with accompanying [`JITModule`].
/// Only call this function if it is safe.
/// Also return i32 is not assumed to be a  valid [`ReactionResult`].
pub type ReactionFn = unsafe extern "C" fn(&mut f32x16) -> ReactionResult;

/// Indicates whether a reaction has occured.
/// Granted, this is not a promise that the mixtures was not affected.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionResult {
    /// Reaction did not occur.
    DidNotReact = 0,
    /// Reaction has occured.
    Reacted = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ROperation {
    Add(String, String, String),
    Sub(String, String, String),
    Mul(String, String, String),
    Div(String, String, String),
    Jump(String),
}

#[derive(Debug, Clone)]
struct RBlock {
    pub name: String,
    pub operations: Vec<ROperation>,
}

type BlockCollection = Vec<RBlock>;

/// Where Reactions are stored.
pub struct ReactionRegistry {
    _module: JITModule,
    /// Reaction functions.
    pub reactions: Vec<ReactionFn>,
}

/// Constructs reactions and returns their functions ready for execution.
pub fn parse_and_build_reactions(
    reaction_prototypes: Vec<(&str, &str)>,
) -> Result<ReactionRegistry, Box<dyn Error>> {
    let mut parsed_reactions = Vec::with_capacity(reaction_prototypes.len());
    for (_name_prototype, reaction_prototype) in reaction_prototypes.iter() {
        let parsed = parse_reaction(*reaction_prototype)?;
        parsed_reactions.push(parsed);
    }

    let (module, reactions) = build_reactions(parsed_reactions)?;

    let registry = ReactionRegistry { _module: module, reactions };

    Ok(registry)
}
