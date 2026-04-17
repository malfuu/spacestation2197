//! Content Loading in Space Station 2197
//! Done with assistance of lua.
//! Wallahi, in the future, we can fully switch to bsn
pub mod entity;
mod environment;
pub mod prototype;
mod runtime;

pub mod prelude;

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
            .insert_resource(ContentPath(self.script.clone()))
            .add_systems(PreStartup, load);

        app.add_plugins(ContentEntityPlugin);
    }
}

#[derive(Resource, Deref)]
pub struct ContentHash(u32);

#[derive(Resource, Clone)]
struct ContentPath(String);

fn load(world: &mut World) {
    let path_str = world.resource::<ContentPath>().0.clone();
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
