use std::collections::HashMap;

use bevy::prelude::*;
use mlua::prelude::*;

use indextree::{Arena, NodeId};

use crate::{ParserRegistry, entity::COMPONENTS_KEY, prototype::Prototypes, runtime::Defines};

struct RawPrototype {
    id: String,
    parent: String,
    is_abstract: bool,
    table: LuaTable,
}

#[derive(Default, Deref, DerefMut)]
struct RawPrototypeCollection(HashMap<String, Vec<RawPrototype>>);

pub(super) fn build_prototypes(
    lua: &Lua,
    parser_registry: &ParserRegistry,
    defines: &Defines,
) -> Option<Prototypes> {
    let mut prototype_collection = build_raw_prototypes(defines);
    apply_inheritances(lua, &mut prototype_collection);
    let prototypes = create_prototypes(lua, parser_registry, &prototype_collection);

    Some(prototypes)
}

fn build_raw_prototypes(defines: &Defines) -> RawPrototypeCollection {
    let mut collection = RawPrototypeCollection::default();

    for (category, tables) in defines.iter() {
        let mut raw_prototypes = Vec::with_capacity(tables.len());

        for table in tables {
            let id: String = table.get("id").unwrap_or_default();
            let parent: String = table.get("parent").unwrap_or_default();

            let is_abstract: bool = table.get("abstract").unwrap_or(false);

            raw_prototypes.push(RawPrototype {
                id,
                parent,
                is_abstract,
                table: table.clone(),
            });
        }

        collection.insert(category.clone(), raw_prototypes);
    }

    collection
}

fn apply_inheritances(lua: &Lua, prototypes: &mut RawPrototypeCollection) {
    for proto_list in prototypes.values() {
        let (arena, nodes) = build_inheritance_tree(proto_list);
        apply_tree_inheritance(lua, proto_list, &arena, &nodes);
    }
}

fn build_inheritance_tree(proto_list: &[RawPrototype]) -> (Arena<usize>, Vec<NodeId>) {
    let mut arena = Arena::new();
    let mut id_to_node = HashMap::new();

    let nodes: Vec<NodeId> = proto_list
        .iter()
        .enumerate()
        .map(|(i, proto)| {
            let node = arena.new_node(i);
            if !proto.id.is_empty() {
                id_to_node.insert(proto.id.clone(), node);
            }
            node
        })
        .collect();

    for (i, proto) in proto_list.iter().enumerate() {
        if let Some(&parent_node) = id_to_node.get(&proto.parent) {
            parent_node.append(nodes[i], &mut arena);
        }
    }

    (arena, nodes)
}

fn apply_tree_inheritance(
    lua: &Lua,
    raw_prototype_list: &[RawPrototype],
    arena: &Arena<usize>,
    nodes: &[NodeId],
) {
    for &node in nodes {
        if arena.get(node).unwrap().parent().is_some() {
            continue;
        }

        for desc in node.descendants(arena) {
            if let Some(parent_node) = arena.get(desc).unwrap().parent() {
                let child_idx = *arena.get(desc).unwrap().get();
                let parent_idx = *arena.get(parent_node).unwrap().get();

                apply_table_inheritance(
                    lua,
                    &raw_prototype_list[parent_idx].table,
                    &raw_prototype_list[child_idx].table,
                )
                .expect("Could not apply inheritance");
            }
        }
    }
}

fn apply_table_inheritance(
    lua: &Lua,
    parent_table: &LuaTable,
    child_table: &LuaTable,
) -> LuaResult<()> {
    for pair in parent_table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;

        // exception: components
        if let Some(key) = key.as_string()
            && key.to_string_lossy() == COMPONENTS_KEY
        {
            let parent_components = parent_table.get::<LuaTable>(COMPONENTS_KEY)?;
            let Ok(child_components) = child_table.get::<LuaTable>(COMPONENTS_KEY) else {
                // kid with no components? hardset with parent's
                child_table.set(key, value)?;
                continue;
            };

            apply_component_inheritance(lua, &parent_components, &child_components)
                .expect("Could not apply component inheritance");

            continue;
        }

        if !child_table.contains_key(key.clone())? {
            child_table.set(key, value)?;
        }
    }

    Ok(())
}

use mlua::{Lua, Result as LuaResult, Table, Value};
use std::collections::HashSet;

pub fn apply_component_inheritance(
    _lua: &Lua,
    parent_components: &Table,
    child_components: &Table,
) -> LuaResult<()> {
    let mut existing_child_components = HashSet::new();

    let mut max_child_index: i64 = 0;

    for pair in child_components.pairs::<Value, Value>() {
        let (key, value) = pair?;

        match (key, &value) {
            (Value::Integer(idx), Value::String(comp_name)) => {
                existing_child_components.insert(comp_name.to_str()?.to_string());
                if idx > max_child_index {
                    max_child_index = idx;
                }
            }
            (Value::String(comp_name), Value::Table(_)) => {
                existing_child_components.insert(comp_name.to_str()?.to_string());
            }
            _ => {}
        }
    }

    for pair in parent_components.pairs::<Value, Value>() {
        let (key, value) = pair?;

        match (key, value) {
            (Value::Integer(_), Value::String(comp_name)) => {
                let name_str = comp_name.to_str()?.to_string();

                if !existing_child_components.contains(&name_str) {
                    max_child_index += 1;
                    child_components.set(max_child_index, comp_name)?;
                    existing_child_components.insert(name_str);
                }
            }
            (Value::String(comp_name), Value::Table(comp_table)) => {
                let name_str = comp_name.to_str()?.to_string();

                if !existing_child_components.contains(&name_str) {
                    child_components.set(comp_name, comp_table)?;
                    existing_child_components.insert(name_str);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn create_prototypes(
    lua: &Lua,
    registry: &ParserRegistry,
    prototypes: &RawPrototypeCollection,
) -> Prototypes {
    let mut final_prototypes = Prototypes::default();

    for (category, proto_list) in prototypes.iter() {
        if let Some(parser) = registry.parsers.get(category) {
            for proto in proto_list {
                if proto.is_abstract {
                    continue;
                }

                match parser(lua, proto.table.clone()) {
                    Ok(boxed) => {
                        final_prototypes.add(category.clone(), proto.id.clone(), boxed);
                    }
                    Err(err) => {
                        error!(
                            "Failed to parse prototype '{}' in category '{}': {}",
                            proto.id, category, err
                        );
                    }
                }
            }
        } else {
            warn!("No parser registered for prototype category: {}", category);
        }
    }

    final_prototypes
}
