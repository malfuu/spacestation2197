//! Stuff that hasn't been sorted into a module yet, or is temporary.
pub mod chat;

use bevy::{prelude::*, world_serialization::WorldInstanceReady};

use shared::game::mob::color::SkinColor;

pub(super) struct ClientPlaceholderPlugin;

impl Plugin for ClientPlaceholderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_color_add);
    }
}

fn on_color_add(
    add: On<WorldInstanceReady>,
    skin_colors: Query<&SkinColor>,
    children_query: Query<&Children>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut commands: Commands,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(color_variable) = skin_colors.get(add.entity) else {
        return;
    };

    let new_color = Color::Srgba(color_variable.0);

    for descendant in children_query.iter_descendants(add.entity) {
        let Ok(material_handle) = mesh_materials.get(descendant) else {
            continue;
        };

        let new_material = {
            let Some(material) = asset_materials.get(material_handle.id()) else {
                continue;
            };
            let mut new_material = material.clone();
            new_material.base_color = new_color;
            new_material
        };

        commands
            .entity(descendant)
            .insert(MeshMaterial3d(asset_materials.add(new_material)));
    }
}
