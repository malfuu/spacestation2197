//! Atmospheric primitives for simulation.
#![deny(missing_docs)]

/// Gas list management and registry storage.
pub mod gas_list;
/// Gas Mixture and some mathematical definitions.
pub mod gas_mixture;
/// Blueprints for spawning predefined gas mixtures.
pub mod mixture_blueprint;

#[doc(hidden)]
pub mod prelude;

use bevy::prelude::*;

/// Ideal gas law constant at J K^-1 mol^-1
pub const IDEAL_GAS_CONSTANT: f32 = 8.314_463;
/// Required moles to avoid being culled.
pub const MINIMUM_AMOUNT_OF_SUBSTANCE: f32 = 1e-4;
/// In kilopascals
pub const MINIMUM_DELTA_PRESSURE: f32 = 1e-4;
/// Wind force
pub const NEWTONS_PER_KILOPASCAL: f32 = 30.0; // arbitrary btw
/// Coefficient on the notion that air can travel 4 directions or remain still, ergo 5
pub const BASE_DIFFUSION_COEFFICIENT: f32 = 1.0 / 5.0;

/// A unique identifier for a specific gas type, represented as an index.
pub type GasId = usize;

/// Controls the sizes of arrays in the entire simulation.
pub const MAX_NUMBER_OF_GASES: usize = 16;
/// Arrays that contain a property per [`Gas`] indexed by [`GasId`]
pub type PerGasArray = [f32; MAX_NUMBER_OF_GASES];

/// Creates a [`PerGasArray`] given an amount.
pub fn per_gas_array(amount: f32) -> PerGasArray {
    [amount; MAX_NUMBER_OF_GASES]
}

/// Returns an iterator over all gas identifiers up to [`MAX_NUMBER_OF_GASES`].
/// These gas identifiers might not map to an existing gas type.
pub fn iter_gas_ids() -> impl Iterator<Item = usize> {
    0..MAX_NUMBER_OF_GASES
}

/// Asserts that a given [`GasId`] is within the valid range.
pub fn assert_gas_id(gas_id: GasId) {
    assert!(gas_id < MAX_NUMBER_OF_GASES,);
}

/// Defines the properties of a gas type.
#[derive(Debug, Clone)]
pub struct Gas {
    /// Name for the gas.
    pub name: String,
    /// Molar Heat Capacity for the gas, in joule per mole kelvin
    /// Bigger values require more energy to increase temperature by one Kelvin.
    pub molar_heat_capacity: f32,
}

impl Gas {
    /// Creates a new [`Gas`] instance.
    pub fn new(name: String, molar_heat_capacity: f32) -> Self {
        Self {
            name,
            molar_heat_capacity,
        }
    }
}
