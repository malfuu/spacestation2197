//! Content Loading of user-defined prototypes.
//! Content is done manually with the [`load`] function,
//! Performing a stop-the-world load by running lua scripts [``] to collect all prototype
//! definitions. Lua script entry point by default is set to [`DEFAULT_SCRIPT`].
//! In the context of the project overall, despite the fact that both `server` and `client` load
//! files, only the `server` spawns prototyped entities, meaning any client side defined component
//! is not added to spawned to replicated entities.
//! Also, we have a [`ContentHash`] to verify the state of what prototype definitions were loaded.
//! (not the asset files themselves)
//! Wallahi, in the future, we can fully switch to dynamic BSN.
pub mod entity;
mod environment;
pub mod prototype;
mod runtime;

pub mod prelude;

use std::fmt;
use std::path::Path;

use bevy::prelude::*;

use crate::{
    entity::{ContentEntityPlugin, component::ComponentPrototypes},
    prototype::ParserRegistry,
    runtime::load_content,
};

const DEFAULT_SCRIPT: &str = "assets/data.lua";

pub struct ContentPlugin {
    script: String,
}

impl ContentPlugin {
    pub fn new(script: impl Into<String>) -> Self {
        Self {
            script: script.into(),
        }
    }
}

impl Default for ContentPlugin {
    fn default() -> Self {
        Self::new(DEFAULT_SCRIPT)
    }
}

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<ParserRegistry>()
            .insert_resource(ContentEntryPointPath(self.script.clone()))
            .add_systems(PreStartup, load.in_set(LoadContentSystems));

        app.add_plugins(ContentEntityPlugin);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadContentSystems;

#[derive(Resource, Deref, Debug)]
pub struct ContentHash(u32);

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

#[derive(Resource, Clone)]
struct ContentEntryPointPath(String);

fn load(world: &mut World) {
    let path_str = world.resource::<ContentEntryPointPath>().0.clone();
    let path = Path::new(&path_str);

    let parser_registry = world
        .remove_non_send_resource::<ParserRegistry>()
        .expect("ParserRegistry should be initialized before loading");

    let component_prototypes = world
        .remove_resource::<ComponentPrototypes>()
        .expect("Component Prototypes should be initialized before loading");

    let (prototypes, hasher) = load_content(parser_registry, component_prototypes, path);
    let content_hash = ContentHash(hasher.finalize());

    world.insert_resource(prototypes);
    world.insert_resource(content_hash);
}
