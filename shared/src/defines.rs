//! Series of constants used throughout the game
use bevy::math::prelude::*;

pub use content::prelude::PROTOTYPE_CATEGORY_ENTITY;

pub const NAME_FULL: &'static str = "Space Station 2197";
pub const NAME_SHORT: &'static str = "ss2197";

pub const DEFAULT_LISTEN_PORT: u16 = 2197; // NOTE: might cause collision with Apple's APNs

pub const DEFAULT_TPS: f64 = 30.0;

// SIZE DIMENSIONS
pub const CEILING_HEIGHT: f32 = 2.5;
pub const CEILING_HEIGHT_HALF: f32 = CEILING_HEIGHT / 2.0;

/// Cuboid size of the above-surface volume belonging to a tile.
pub const TILE_CUBOID: Vec3 = Vec3::new(1.0, CEILING_HEIGHT, 1.0);
pub const TILE_VOLUME: f32 = TILE_CUBOID.x * TILE_CUBOID.y * TILE_CUBOID.z;

pub const MOB_REACH: f32 = 1.0;
pub const SAY_REACH: f32 = 16.0;

pub const PROTOTYPE_TYPE_TILE: &str = "tile";
