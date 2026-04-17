use std::{any::Any, collections::HashMap};

use bevy::prelude::*;
use mlua::prelude::*;

pub type BoxedPrototype = Box<dyn Any + Send + Sync>;

#[derive(Default)]
struct Categories(HashMap<String, BoxedPrototype>);

#[derive(Resource, Default)]
pub struct Prototypes(HashMap<String, Categories>);

impl Prototypes {
    pub fn add(&mut self, category: impl Into<String>, id: String, prototype: BoxedPrototype) {
        self.0
            .entry(category.into())
            .or_default()
            .0
            .insert(id, prototype);
    }

    pub fn get<T: 'static>(&self, category: &str, id: impl Into<String>) -> Option<&T> {
        self.0.get(category)?.0.get(&id.into())?.downcast_ref::<T>()
    }

    pub fn iter_for_category<T: 'static>(&self, category: &str) -> impl Iterator<Item = &T> {
        self.0
            .get(category)
            .into_iter()
            .flat_map(|cat| cat.0.values())
            .filter_map(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn iter_for_category_entries(&self, category: &str) -> impl Iterator<Item = &String> {
        self.0
            .get(category)
            .into_iter()
            .flat_map(|cat| cat.0.keys())
    }

    pub fn contains(&self, category: &str, id: &String) -> bool {
        self.0
            .get(category)
            .is_some_and(|cat| cat.0.contains_key(id))
    }
}

pub type ParseResult = LuaResult<BoxedPrototype>;
pub type PrototypeParseFn = Box<dyn Fn(&Lua, LuaTable) -> ParseResult>;

#[derive(Default)]
pub(super) struct ParserRegistry {
    pub(super) parsers: HashMap<String, PrototypeParseFn>,
}

pub trait PrototypeAppExt {
    fn prototype<T>(
        &mut self,
        category: impl Into<String>,
        parser: impl Fn(&Lua, LuaTable) -> ParseResult + 'static,
    ) -> &mut Self;
}

impl PrototypeAppExt for App {
    fn prototype<T>(
        &mut self,
        category: impl Into<String>,
        parser: impl Fn(&Lua, LuaTable) -> ParseResult + 'static,
    ) -> &mut Self {
        let mut registry = self.world_mut().non_send_resource_mut::<ParserRegistry>();
        registry.parsers.insert(category.into(), Box::new(parser));

        self
    }
}
