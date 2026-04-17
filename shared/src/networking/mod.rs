use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy_replicon_renet::RepliconRenetPlugins;
use common::EntityTag;

pub const NETWORK_PROTOCOL_ID: u64 = 0;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins)
            .replicate::<Name>()
            .replicate::<Transform>()
            .sync_related_entities::<ChildOf>()
            .replicate::<ChildOf>()
            .replicate_once::<EntityTag>();
    }
}
