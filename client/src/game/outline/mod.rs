use bevy::prelude::*;

// use bevy_mod_outline::{
//     AsyncSceneInheritOutline, AutoGenerateOutlineNormalsPlugin, OutlineMode, OutlinePlugin,
//     OutlineVolume,
// };
// use bevy_replicon::prelude::*;
//
// use shared::{
//     defines::MOB_REACH,
//     game::{hands::Hands, interact::Interactable},
// };
//
// use crate::{
//     base::{grid::Tile, input::ExtraInputs},
//     game::mind::Controlling,
// };
//
// pub const INTERACT_COLOR: Color = Color::srgba(0.0, 1.0, 0.0, 0.4);
// pub const INTERACT_WIDTH: f32 = 1.5;
//
// #[derive(QueryFilter)]
// pub struct OutlinableFilter {
//     replicateds: With<Remote>,
//     interactables: With<Interactable>,
//     no_tiles: Without<Tile>,
// }
//
// #[derive(Resource, Default, Debug, Deref, DerefMut)]
// pub struct HoveredEntity(pub Option<Entity>);
//
pub(super) struct ClientOutlinePlugin;

impl Plugin for ClientOutlinePlugin {
    fn build(&self, app: &mut App) {
        // app.init_resource::<HoveredEntity>()
        //     .add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin::default()))
        //     .add_systems(Update, (handle_hover_state, update_outline).chain());
    }
}
//
// fn handle_hover_state(
//     mut commands: Commands,
//     extra_inputs: Res<ExtraInputs>,
//     mut hovered_entity: ResMut<HoveredEntity>,
//     outlinable: Query<Entity, OutlinableFilter>,
// ) {
//     if hovered_entity.0 == extra_inputs.hovering() {
//         return;
//     }
//
//     if let Some(old_entity) = hovered_entity.0
//         && let Ok(mut entity_commands) = commands.get_entity(old_entity)
//     {
//         entity_commands.remove::<OutlineVolume>();
//     }
//
//     let Some(new_entity) = extra_inputs.hovering() else {
//         hovered_entity.0 = None;
//         return;
//     };
//
//     if outlinable.contains(new_entity) {
//         commands.entity(new_entity).insert((
//             OutlineVolume {
//                 visible: false,
//                 width: INTERACT_WIDTH,
//                 colour: INTERACT_COLOR,
//             },
//             AsyncSceneInheritOutline::default(),
//             OutlineMode::FloodFlatDoubleSided,
//         ));
//
//         hovered_entity.0 = Some(new_entity);
//     }
// }
//
// pub type ControllingSingle<'w, 's> =
//     Single<'w, 's, (&'static Transform, Option<&'static Hands>), With<Controlling>>;
//
// fn update_outline(
//     controlled: Option<ControllingSingle>,
//     transforms: Query<&Transform>,
//     hovered_entity: Res<HoveredEntity>,
//     mut outline_volumes: Query<&mut OutlineVolume>,
// ) {
//     let Some(hovered) = **hovered_entity else {
//         return;
//     };
//
//     let Ok(mut outline) = outline_volumes.get_mut(hovered) else {
//         return;
//     };
//
//     let mut visible = false;
//
//     if let Some(controlled) = controlled {
//         let (mob_transform, mob_hands_opt) = controlled.into_inner();
//
//         if let Ok(hovered_transform) = transforms.get(hovered)
//             && mob_hands_opt.is_some()
//         {
//             let distance = mob_transform
//                 .translation
//                 .distance(hovered_transform.translation);
//
//             visible |= distance < MOB_REACH;
//         }
//     }
//
//     outline.visible = visible;
// }
