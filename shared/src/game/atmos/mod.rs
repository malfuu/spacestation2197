use bevy::prelude::*;
use mlua::prelude::*;

use bevy_replicon::prelude::*;

use common::PrototypeId;

use atmos_primitives::prelude::*;
use atmos_primitives::reactions::{
    ReactionInformation, ReactionModel, ReactionRegistry, parse_and_build_reactions,
};
use atmos_simulation::prelude::*;
use content::prelude::*;
use serde::{Deserialize, Serialize};

pub const PROTOTYPE_TYPE_GAS: &str = "gas";
pub const PROTOTYPE_TYPE_MIXTURE: &str = "gas_mixture";
pub const PROTOTYPE_TYPE_REACTION: &str = "reaction";

pub(super) struct AtmosPlugin;

impl Plugin for AtmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AtmosphericsPlugin)
            .prototype::<GasPrototype>(PROTOTYPE_TYPE_GAS, gas_parser)
            .prototype::<ReactionPrototype>(PROTOTYPE_TYPE_REACTION, reaction_parser)
            .prototype::<MixturePrototype>(PROTOTYPE_TYPE_MIXTURE, mixture_parser)
            .replicate::<Excited>()
            .replicate::<ChunkMixtures>()
            // .replicate::<Flows>()
            .add_systems(Startup, load_gas_protos);
    }
}

fn build_gas_list(prototype_list: &Prototypes) -> GasList {
    let gases = prototype_list
        .iter_for_category::<GasPrototype>(PROTOTYPE_TYPE_GAS)
        .map(
            |GasPrototype {
                 id,
                 gas_id,
                 molar_heat_capacity,
                 color: _,
             }| Gas {
                gas_id: *gas_id,
                name: id.clone(),
                molar_heat_capacity: *molar_heat_capacity,
            },
        )
        .collect();

    GasList::new(gases)
}

fn build_mixture_list(prototype_list: &Prototypes, gas_list: &GasList) -> MixtureTemplateList {
    let mut mixture_list = MixtureTemplateList::new();

    prototype_list
        .iter_for_category::<MixturePrototype>(PROTOTYPE_TYPE_MIXTURE)
        .for_each(|proto| {
            let ratios = proto
                .ratios
                .iter()
                .map(|(name, amount)| {
                    let gas_id = gas_list.try_get_gas_id_by_name(name).unwrap_or_else(|| {
                        panic!("Invalid gas name: {} found in mixture {}!", name, proto.id)
                    });
                    (gas_id, *amount)
                })
                .collect();

            let template =
                MixtureTemplate::new(proto.id.clone(), proto.pressure, proto.temperature, ratios);

            mixture_list.add(template);
        });

    mixture_list
}

fn build_reaction_registry(prototype_list: &Prototypes, gas_list: &GasList) -> ReactionRegistry {
    let mut reaction_models = Vec::new();

    for proto in prototype_list.iter_for_category::<ReactionPrototype>(PROTOTYPE_TYPE_REACTION) {
        let mut required_gases_array = [0.0f32; 16];
        for (gas_name, amount) in &proto.required_gases {
            if let Some(gas_id) = gas_list.try_get_gas_id_by_name(gas_name) {
                if gas_id < MAX_NUMBER_OF_GASES {
                    required_gases_array[gas_id] = *amount;
                }
            } else {
                panic!(
                    "Invalid gas name: {} found in reaction {} required gases!",
                    gas_name, proto.id
                );
            }
        }

        let required_gases = PerGasArray::new(required_gases_array);

        reaction_models.push(ReactionModel {
            information: ReactionInformation {
                name: proto.id.clone(),
                priority: proto.priority,
                required_gases,
            },
            function: proto.code.clone(),
        });
    }

    parse_and_build_reactions(reaction_models, gas_list).unwrap()
}

pub(crate) fn load_gas_protos(world: &mut World) {
    let protos = world.resource::<Prototypes>();
    let gas_list = build_gas_list(protos);
    let mixture_list = build_mixture_list(protos, &gas_list);
    let reaction_registry = build_reaction_registry(protos, &gas_list);

    world.insert_resource(gas_list);
    world.insert_resource(mixture_list);
    world.insert_non_send(reaction_registry);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrototype {
    pub id: PrototypeId,
    pub gas_id: GasId,
    pub molar_heat_capacity: f32,
    pub color: Srgba,
}

pub fn gas_parser(_: &Lua, table: LuaTable) -> ParseResult {
    let proto = GasPrototype {
        id: table.get("id")?,
        gas_id: table.get("gas_id")?,
        molar_heat_capacity: table.get("molar_heat_capacity")?,
        color: Srgba::new(1.0, 1.0, 1.0, 1.0),
    };

    Ok(Box::new(proto))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixturePrototype {
    pub id: PrototypeId,
    pub pressure: f32,
    pub temperature: f32,
    pub ratios: Vec<(String, f32)>,
}

pub fn mixture_parser(_: &Lua, table: LuaTable) -> ParseResult {
    let ratios_table = table.get::<LuaTable>("ratios")?;

    let mut ratios = Vec::new();
    for pair in ratios_table.pairs::<String, f32>() {
        let gas_and_ratio = pair?;
        ratios.push(gas_and_ratio);
    }

    let proto = MixturePrototype {
        id: table.get("id")?,
        pressure: table.get("pressure")?,
        temperature: table.get("temperature")?,
        ratios,
    };

    Ok(Box::new(proto))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionPrototype {
    pub id: PrototypeId,
    pub priority: i32,
    pub required_gases: Vec<(String, f32)>,
    pub code: String,
}

pub fn reaction_parser(_: &Lua, table: LuaTable) -> ParseResult {
    let mut required_gases = Vec::new();

    if let Ok(required_gases_table) = table.get::<LuaTable>("required_gases") {
        for pair in required_gases_table.pairs::<String, f32>() {
            let (gas_name, amount) = pair?;
            required_gases.push((gas_name, amount));
        }
    }

    let proto = ReactionPrototype {
        id: table.get("id")?,
        priority: table.get("priority").unwrap_or(0i32),
        required_gases,
        code: table.get("code")?,
    };

    Ok(Box::new(proto))
}
