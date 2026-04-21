use bevy::prelude::*;

// these Enter structs will disappear until we implement actual channels:

#[derive(Event, Debug, Deref)]
pub struct OocEnter(pub String);

#[derive(Event, Debug, Deref)]
pub struct SayEnter(pub String);
