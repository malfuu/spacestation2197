pub mod gas_list;
pub mod gas_mixture;
pub mod mixture_blueprint;

#[doc(hidden)]
pub mod prelude;

use bevy::prelude::*;

use uom::si::{f32::*, molar_heat_capacity::joule_per_kelvin_mole};

pub const IDEAL_GAS_CONSTANT: f32 = 8.314_463;

/// Controls the sizes of arrays in the entire simulation.
pub const MAX_NUMBER_OF_GASES: usize = 16;
/// Required moles to avoid being culled.
pub const MINIMUM_AMOUNT_OF_SUBSTANCE: f32 = 1e-4;
/// In kilopascals
pub const MINIMUM_DELTA_PRESSURE: f32 = 1e-4;
/// Wind force
pub const NEWTONS_PER_KILOPASCAL: f32 = 30.0; // arbitrary btw
/// Coefficient on the notion that air can travel 4 directions or remain still, ergo 5
pub const BASE_DIFFUSION_COEFFICIENT: f32 = 1.0 / 5.0;

pub type GasId = usize;

pub fn iter_gas_ids() -> impl Iterator<Item = usize> {
    0..MAX_NUMBER_OF_GASES
}

pub fn assert_gas_id(gas_id: GasId) {
    assert!(gas_id < MAX_NUMBER_OF_GASES,);
}

#[derive(Debug, Clone)]
pub struct Gas {
    pub name: String,
    pub molar_heat_capacity: MolarHeatCapacity,
}

impl Gas {
    pub fn new(name: String, molar_heat_capacity: f32) -> Gas {
        Gas {
            name,
            molar_heat_capacity: MolarHeatCapacity::new::<joule_per_kelvin_mole>(
                molar_heat_capacity,
            ),
        }
    }
}
