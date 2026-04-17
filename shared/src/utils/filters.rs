//! Also, these filters work on the assumption that they
//! actually result in better performance

use bevy::{ecs::query::QueryFilter, prelude::*};

use grid::{Chunk, Grid};

use crate::{
    game::{items::Item, markers::Marker, mob::Mob, player::Player},
    meta::manager::Manager,
};

#[derive(QueryFilter)]
pub struct GridFilter {
    with: With<Grid>,
    // grid
    no_chunks: Without<Chunk>,
    // entities
    no_mobs: Without<Mob>,
    no_items: Without<Item>,
    no_markers: Without<Marker>,
    // meta
    no_manager: Without<Manager>,
    no_players: Without<Player>,
}

#[derive(QueryFilter)]
pub struct ChunkFilter {
    with: With<Chunk>,
    // grid
    no_grids: Without<Grid>,
    // entities
    no_mobs: Without<Mob>,
    no_items: Without<Item>,
    no_markers: Without<Marker>,
    // meta
    no_manager: Without<Manager>,
    no_players: Without<Player>,
}

#[derive(QueryFilter)]
pub struct MobFilter {
    with: With<Mob>,
    // grid
    no_grids: Without<Grid>,
    no_chunks: Without<Chunk>,
    // entities
    no_items: Without<Item>,
    no_markers: Without<Marker>,
    // meta
    no_manager: Without<Manager>,
    no_players: Without<Player>,
}

#[derive(QueryFilter)]
pub struct ItemFilter {
    with: With<Item>,
    // grid
    no_grids: Without<Grid>,
    no_chunks: Without<Chunk>,
    // entities
    no_mobs: Without<Mob>,
    no_markers: Without<Marker>,
    // meta
    no_manager: Without<Manager>,
    no_players: Without<Player>,
}

#[derive(QueryFilter)]
pub struct MarkerFilter {
    with: With<Marker>,
    // grid
    no_grids: Without<Grid>,
    no_chunks: Without<Chunk>,
    // entities
    no_mobs: Without<Mob>,
    no_items: Without<Item>,
    // meta
    no_manager: Without<Manager>,
    no_players: Without<Player>,
}

#[derive(QueryFilter)]
pub struct ManagerFilter {
    with: With<Manager>,
    // grid
    no_grids: Without<Grid>,
    no_chunks: Without<Chunk>,
    // entities
    no_mobs: Without<Mob>,
    no_items: Without<Item>,
    no_markers: Without<Marker>,
    // meta
    no_players: Without<Player>,
}

#[derive(QueryFilter)]
pub struct PlayerFilter {
    with: With<Player>,
    // grid
    no_grids: Without<Grid>,
    no_chunks: Without<Chunk>,
    // entities
    no_mobs: Without<Mob>,
    no_items: Without<Item>,
    no_markers: Without<Marker>,
    // meta
    no_manager: Without<Manager>,
}
