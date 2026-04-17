//! fair warning: this crate used to be a lot stuff before
//! but, as I got more organized, stuff got cut from here.
//! god willing, in the future this will probably just be removed

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: investigate turning these into ASCII only 32 byte heapless strings.
pub type PrototypeId = String;

/// Marks entity prototype name
#[derive(Component, Clone, Reflect, Debug, Deref, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component)]
// TODO: tags could get help from hashing, like bevy's Name
pub struct EntityTag(pub PrototypeId);

pub type TileTag = PrototypeId;
