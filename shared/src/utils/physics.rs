use avian3d::prelude::*;
use bevy::prelude::*;

pub trait PhysicsEntityCommandsExt {
    fn enable_physics(&mut self) -> &mut Self;
    fn disable_physics(&mut self) -> &mut Self;
}

impl<'a> PhysicsEntityCommandsExt for EntityCommands<'a> {
    fn enable_physics(&mut self) -> &mut Self {
        self.remove::<ColliderDisabled>()
            .remove::<RigidBodyDisabled>()
            .remove::<JointDisabled>();

        self
    }

    fn disable_physics(&mut self) -> &mut Self {
        self.insert(ColliderDisabled)
            .insert(RigidBodyDisabled)
            .insert(JointDisabled);

        self
    }
}
