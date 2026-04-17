mod builder;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bevy::prelude::*;
use crc32fast::Hasher;
use mlua::{AsChunk, prelude::*};

use crate::{
    ParserRegistry,
    entity::{EntityAppData, component::ComponentPrototypes},
    environment::{CommonAppData, create_environment},
    prelude::Prototypes,
    runtime::builder::build_prototypes,
};

#[derive(Default, Debug, Deref, DerefMut)]
struct Defines(HashMap<String, Vec<LuaTable>>);

#[derive(Default, Clone)]
struct LoaderAppData {
    /// Incrementally saves hashes of loaded scripts.
    defines: Rc<RefCell<Defines>>,
}

pub(super) fn load_content(
    parser_registry: ParserRegistry,
    prototype_components: ComponentPrototypes,
    as_chunk: impl AsChunk,
) -> (Prototypes, Hasher) {
    let lua = Lua::new();
    let common = CommonAppData::default();
    let loader = LoaderAppData::default();
    let entity = EntityAppData::from(prototype_components);
    lua.set_app_data(common.clone());
    lua.set_app_data(loader.clone());
    lua.set_app_data(entity);

    create_environment(&lua).expect("Could not create lua environment.");

    // collect defines
    let chunk = lua.load(as_chunk);
    chunk.exec().expect("Could not execute lua code");

    let defines = loader.defines.borrow();
    let prototypes =
        build_prototypes(&lua, &parser_registry, &defines).expect("Should have prototypes.");

    let hasher = common.hasher.borrow().clone();

    (prototypes, hasher)
}

pub(crate) fn env_define(lua: &Lua, table: LuaTable) -> LuaResult<()> {
    let category_id: String = table.get(1)?;

    let app_data = lua
        .app_data_mut::<LoaderAppData>()
        .expect("LoaderAppData should exist.")
        .clone();

    let mut defines = app_data.defines.borrow_mut();
    defines.entry(category_id).or_default().push(table);

    Ok(())
}
