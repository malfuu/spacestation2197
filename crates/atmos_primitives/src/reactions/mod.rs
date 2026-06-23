//! Reactions of mixtures that do not necessarily follow the laws of physics.
mod builder;
pub(crate) mod parser;

use std::error::Error;

use bevy::prelude::*;

use cranelift_jit::JITModule;

use wide::f32x16;

use crate::{
    PerGasArray,
    reactions::{builder::build_reactions, parser::parse_reaction},
};

/// Reaction Fn applied to mixture values.
/// Filled with unsafe black magic due to its JIT compiled nature.
/// # SAFETY
/// Function lifetime associated with accompanying [`JITModule`].
/// Only call this function if it is safe.
/// Also return i32 is not assumed to be a  valid [`ReactionResult`].
pub type ReactionFn = unsafe extern "C" fn(&mut f32x16) -> ReactionResult;

/// A gas mixture that is reactable.
pub trait Reactable {
    /// Performs a reaction on the mixture.
    fn react(&mut self, reaction: ReactionFn);
}

/// Performs the suite of reactions on a [`Reactable`] by order of priority.
pub fn perform_reactions(mixture: &mut impl Reactable, reaction_registry: &ReactionRegistry) {
    for (_, reaction_fn) in reaction_registry.reactions.iter() {
        mixture.react(*reaction_fn);
    }
}

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

/// Reaction details, including relevant functions.
pub struct ReactionInformation {
    /// The name of the reaction.
    pub name: String,
    /// Lower values get executed first. Same priority will have an arbirtrary order.
    pub priority: i32,
    /// Required gases for this reaction to occur.
    /// If a gas type is not required, it will be `0.0`.
    pub required_gases: PerGasArray,
}

/// Prototype for defining gas reactions.
pub struct ReactionPrototype {
    /// Information about this Reaction
    pub information: ReactionInformation,
    /// reaction DSL code defining a function
    pub function: String,
}

/// Reaction // wow this doc is bad
pub type Reaction = (ReactionInformation, ReactionFn);

/// Where Reactions are stored.
/// Non-Send Data until I confirm the lifetime of reaction functions.
pub struct ReactionRegistry {
    _module: JITModule,
    /// Reaction functions. Ordered by priority.
    pub reactions: Vec<Reaction>,
}

/// Constructs reactions and returns their functions ready for execution.
pub fn parse_and_build_reactions(
    reaction_prototypes: Vec<ReactionPrototype>,
) -> Result<ReactionRegistry, Box<dyn Error>> {
    let mut parsed_reactions = Vec::with_capacity(reaction_prototypes.len());

    for prototype in reaction_prototypes.iter() {
        let parsed = parse_reaction(prototype.function.as_str())?;
        parsed_reactions.push(parsed);
    }

    let (module, reaction_fns) = build_reactions(parsed_reactions)?;

    let mut reactions: Vec<Reaction> = reaction_prototypes
        .into_iter()
        .zip(reaction_fns)
        .map(|(proto, reaction_function)| (proto.information, reaction_function))
        .collect();

    reactions.sort_by_key(|(info, _)| info.priority);

    let registry = ReactionRegistry {
        _module: module,
        reactions,
    };

    Ok(registry)
}
