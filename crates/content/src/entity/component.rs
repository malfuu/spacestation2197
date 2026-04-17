use std::collections::HashMap;

use bevy::prelude::*;
use mlua::prelude::*;
use serde::de::DeserializeOwned;

type DeserializeFn = fn(LuaValue) -> LuaResult<Box<dyn Reflect>>;

pub(super) struct ComponentEntry {
    pub(super) deserialize_fn: DeserializeFn,
    pub(super) default: Option<Box<dyn Reflect>>,
}

#[derive(Resource, Default)]
pub(crate) struct ComponentPrototypes {
    pub(super) components: HashMap<String, ComponentEntry>,
}

fn deserialize_to_reflect<T>(value: LuaValue) -> LuaResult<Box<dyn Reflect>>
where
    T: DeserializeOwned + Component + Reflect,
{
    let deserializer = mlua::serde::Deserializer::new(value);
    let component = T::deserialize(deserializer).map_err(mlua::Error::external)?;
    Ok(Box::new(component))
}

impl ComponentPrototypes {
    pub(crate) fn register<T>(&mut self, name: impl Into<String>, default: Option<T>)
    where
        T: DeserializeOwned + Component + Reflect + Clone,
    {
        let default = default.map(|d| Box::new(d) as Box<dyn Reflect>);

        self.components.insert(
            name.into(),
            ComponentEntry {
                default,
                deserialize_fn: deserialize_to_reflect::<T>,
            },
        );
    }

    pub(crate) fn deserialize(
        &self,
        component_name: &str,
        value: LuaValue,
    ) -> LuaResult<Option<Box<dyn Reflect>>> {
        let Some(entry) = self.components.get(component_name) else {
            return Ok(None);
        };

        let component = (entry.deserialize_fn)(value)?;
        Ok(Some(component))
    }
}

pub trait PrototypeComponentAppExt {
    fn prototype_component<T>(&mut self) -> &mut Self
    where
        T: Component + Reflect + TypePath + Clone + DeserializeOwned + Default;

    fn prototype_component_no_default<T>(&mut self) -> &mut Self
    where
        T: Component + Reflect + TypePath + Clone + DeserializeOwned;

    fn prototype_component_with_default<T>(&mut self, default: T) -> &mut Self
    where
        T: Component + Reflect + TypePath + Clone + DeserializeOwned;
}

impl PrototypeComponentAppExt for App {
    fn prototype_component<T>(&mut self) -> &mut Self
    where
        T: Component + Reflect + TypePath + Clone + DeserializeOwned + Default,
    {
        self.prototype_component_with_default(T::default())
    }

    fn prototype_component_no_default<T>(&mut self) -> &mut Self
    where
        T: Component + Reflect + TypePath + Clone + DeserializeOwned,
    {
        register_internal::<T>(self, None)
    }

    fn prototype_component_with_default<T>(&mut self, default: T) -> &mut Self
    where
        T: Component + Reflect + TypePath + Clone + DeserializeOwned,
    {
        register_internal::<T>(self, Some(default))
    }
}

fn register_internal<T>(app: &mut App, default: Option<T>) -> &mut App
where
    T: Component + Reflect + TypePath + Clone + DeserializeOwned,
{
    let mut components = app.world_mut().resource_mut::<ComponentPrototypes>();
    let name = T::short_type_path();

    components.register(name, default);

    app
}
