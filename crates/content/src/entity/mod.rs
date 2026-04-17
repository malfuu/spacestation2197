pub mod component;

use std::{cell::RefCell, rc::Rc};

use bevy::prelude::*;
use mlua::prelude::*;

use common::EntityTag;

use crate::{
    entity::component::ComponentPrototypes,
    prelude::{PrototypeAppExt, Prototypes},
    prototype::ParseResult,
};

pub const PROTOTYPE_CATEGORY_ENTITY: &str = "entity";
pub const COMPONENTS_KEY: &str = "components";

pub struct EntityPrototype {
    pub id: String,
    pub mesh: String,
    pub components: Vec<Box<dyn Reflect>>,
}

pub(super) struct ContentEntityPlugin;

impl Plugin for ContentEntityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComponentPrototypes>()
            .prototype::<EntityPrototype>(PROTOTYPE_CATEGORY_ENTITY, parse_entity);
    }
}

#[derive(Default, Clone)]
pub(super) struct EntityAppData {
    components: Rc<RefCell<ComponentPrototypes>>,
}

impl From<ComponentPrototypes> for EntityAppData {
    fn from(prototypes: ComponentPrototypes) -> Self {
        Self {
            components: Rc::new(RefCell::new(prototypes)),
        }
    }
}

fn parse_entity(lua: &Lua, entity_table: LuaTable) -> ParseResult {
    let id: String = entity_table.get("id").unwrap_or_default();
    let mesh: String = entity_table.get("mesh").unwrap_or_default();

    let binding = lua
        .app_data_ref::<EntityAppData>()
        .expect("EntityAppData should exist!");

    let registry = binding.components.borrow();

    let mut components = Vec::new();
    if let Ok(components_table) = entity_table.get::<LuaTable>("components") {
        components = parse_components(&registry, components_table)?;
    }

    Ok(Box::new(EntityPrototype {
        id,
        mesh,
        components,
    }))
}

fn parse_components(
    registry: &ComponentPrototypes,
    components_table: LuaTable,
) -> LuaResult<Vec<Box<dyn Reflect>>> {
    let mut components = Vec::new();

    for pair in components_table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;

        match (key, value) {
            (LuaValue::Integer(_), LuaValue::String(comp_name)) => {
                let name_str = comp_name.to_str()?;

                if let Some(entry) = registry.components.get(name_str.as_ref()) {
                    if let Some(default_reflect) = &entry.default {
                        let cloned_default = default_reflect
                            .reflect_clone()
                            .expect("Could not clone component");
                        components.push(cloned_default);
                    } else {
                        warn!(
                            "Component prototype '{}' exists, but has no default value configured.",
                            name_str
                        );
                    }
                } else {
                    warn!("Component prototype '{}' not found in registry.", name_str);
                }
            }

            (LuaValue::String(comp_name), comp_value) => {
                let name_str = comp_name.to_str()?;

                match registry.deserialize(name_str.as_ref(), comp_value) {
                    Ok(Some(boxed_component)) => components.push(boxed_component),
                    Ok(None) => warn!("Component prototype '{}' not found in registry.", name_str),
                    Err(e) => error!("Failed to parse component '{}': {}", name_str, e),
                }
            }

            _ => {}
        }
    }

    Ok(components)
}

pub trait PrototypeEntityCommandsExt {
    fn spawn_prototype(&mut self, prototype: String, transform: Transform) -> EntityCommands<'_>;
}

impl<'w, 's> PrototypeEntityCommandsExt for Commands<'w, 's> {
    fn spawn_prototype(
        &mut self,
        prototype_id: String,
        transform: Transform,
    ) -> EntityCommands<'_> {
        let mut e = self.spawn((transform, EntityTag(prototype_id.clone())));

        e.queue(move |mut world: EntityWorldMut<'_>| {
            world.resource_scope::<Prototypes, _>(|world, prototypes| {
                if let Some(proto) =
                    prototypes.get::<EntityPrototype>(PROTOTYPE_CATEGORY_ENTITY, &prototype_id)
                {
                    for component_reflect in &proto.components {
                        let component_clone = component_reflect
                            .reflect_clone()
                            .expect("Could not clone proto component");

                        world.insert_reflect(component_clone);
                    }
                } else {
                    warn!("Could not find entity prototype: {}", prototype_id);
                }
            });
        });

        e
    }
}
