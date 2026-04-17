use bevy::prelude::*;
use content::prelude::*;
use serde::Deserialize;

#[derive(Component, Reflect, Clone, Default, Deserialize, Debug, PartialEq)]
#[reflect(Component)]
struct CompA {
    val_1: f32,
    val_2: f32,
}

#[derive(Component, Reflect, Clone, Deserialize, Debug, PartialEq)]
#[reflect(Component)]
struct CompB {
    id: u32,
    total: u32,
}

#[derive(Component, Reflect, Clone, Deserialize, Debug, PartialEq)]
#[reflect(Component)]
struct CompC(f32);

#[test]
fn test_entity_with_components() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(ContentPlugin::new("assets/entity.lua"));

    app.prototype_component::<CompA>()
        .prototype_component_with_default(CompB { id: 1, total: 2 })
        .prototype_component_with_default(CompC(1.0));

    app.update();

    let world = app.world();
    let prototypes = world.resource::<Prototypes>();

    let entity = prototypes
        .get::<EntityPrototype>("entity", "subject")
        .expect("Prototype should exist.");

    assert_eq!(entity.id, "subject");
    assert_eq!(entity.mesh, "model_alpha.glb");
    assert_eq!(entity.components.len(), 3);
}

#[test]
fn test_entity_component_inheritance() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(ContentPlugin::new("assets/entity_inheritance.lua"));

    app.prototype_component::<CompA>()
        .prototype_component_with_default(CompB { id: 1, total: 2 });

    app.update();

    let world = app.world();
    let prototypes = world.resource::<Prototypes>();

    let child_1 = prototypes
        .get::<EntityPrototype>("entity", "child_1")
        .expect("child_1 prototype should exist.");

    let c1_a = child_1
        .components
        .iter()
        .find_map(|c| c.downcast_ref::<CompA>())
        .expect("child_1 should have CompA");
    assert_eq!(c1_a.val_1, 50.0);
    assert_eq!(c1_a.val_2, 100.0);

    let child_2 = prototypes
        .get::<EntityPrototype>("entity", "child_2")
        .expect("child_2 prototype should exist.");

    let c2_a = child_2
        .components
        .iter()
        .find_map(|c| c.downcast_ref::<CompA>())
        .expect("child_2 should have inherited CompA");
    assert_eq!(c2_a.val_1, 0.0);
}
