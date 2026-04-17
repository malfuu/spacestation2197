use std::time::Instant;

use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::engine::AtmosphericsResource;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct AtmosSchedule;

pub(super) fn run_atmos_schedule(world: &mut World) {
    let _ = info_span!("run_atmos_schedule", name = "atmos").entered();

    let mut r = world.resource_mut::<AtmosphericsResource>();
    if !r.enabled {
        return;
    }

    r.current_tick += 1;

    let _start = Instant::now();
    world.run_schedule(AtmosSchedule);
}
