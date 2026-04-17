use bevy::prelude::*;
use content::prelude::*;
use mlua::prelude::*;

const TYPE_A: &str = "TypeA";
const TYPE_B: &str = "TypeB";

struct StructA {
    x: u32,
    y: bool,
}

fn parse_a(_lua: &Lua, table: LuaTable) -> ParseResult {
    let x: u32 = table.get("x")?;
    let y: bool = table.get("y")?;
    Ok(Box::new(StructA { x, y }))
}

#[test]
fn test_simple_prototype() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ContentPlugin::new("assets/simple.lua")))
        .prototype::<StructA>(TYPE_A, Box::new(parse_a));

    app.update();

    let world = app.world();
    let prototypes = world.resource::<Prototypes>();

    let instance = prototypes
        .get::<StructA>(TYPE_A, "instance_1")
        .expect("Prototype instance_1 should exist.");

    assert_eq!(instance.x, 111);
    assert!(instance.y);
}

struct StructB {
    val_1: u32,
    val_2: u32,
    flag: bool,
}

fn parse_b(_: &Lua, table: LuaTable) -> ParseResult {
    let val_1: u32 = table.get("val_1")?;
    let val_2: u32 = table.get("val_2")?;
    let flag: bool = table.get("flag")?;

    Ok(Box::new(StructB { val_1, val_2, flag }))
}

#[test]
fn test_inheritance_and_abstract() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ContentPlugin::new("assets/inheritance.lua")))
        .prototype::<StructB>(TYPE_B, Box::new(parse_b));

    app.update();

    let world = app.world();
    let prototypes = world.resource::<Prototypes>();

    assert!(
        prototypes.get::<StructB>(TYPE_B, "base_abstract").is_none(),
        "Abstract prototypes should not be in the final registry!"
    );

    let child_1 = prototypes
        .get::<StructB>(TYPE_B, "child_level_1")
        .expect("child_level_1 should exist");

    assert_eq!(child_1.val_1, 15);
    assert_eq!(child_1.val_2, 100);
    assert!(!child_1.flag);

    let child_2 = prototypes
        .get::<StructB>(TYPE_B, "child_level_2")
        .expect("child_level_2 should exist");

    assert_eq!(child_2.val_1, 15);
    assert_eq!(child_2.val_2, 100);
    assert!(child_2.flag);
}
