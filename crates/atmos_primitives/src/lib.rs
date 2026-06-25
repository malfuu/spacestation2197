//! Atmospheric primitives for simulation.
#![deny(missing_docs)]

/// Equations for calculating mixture properties.
pub mod equations;
pub mod gas_list;
pub mod gas_mixture;
pub mod hotspot;
pub mod mixture_template;
pub mod reactions;

#[doc(hidden)]
pub mod prelude;

use bevy::prelude::*;
use wide::f32x16;

/// Ideal gas law constant at Joules per Kelvin Mole
pub const IDEAL_GAS_CONSTANT: f32 = 8.314_463;
/// Required moles to avoid being culled.
pub const MINIMUM_AMOUNT_OF_SUBSTANCE: f32 = 1e-8;
/// In Kilopascals
pub const MINIMUM_DELTA_PRESSURE: f32 = 1e-1;
/// Wind force
pub const NEWTONS_PER_KILOPASCAL: f32 = 30.0; // arbitrary btw
/// Coefficient on the notion that air can travel 4 directions or remain still, ergo 5
pub const BASE_DIFFUSION_COEFFICIENT: f32 = 1.0 / 5.0;

/// A unique identifier for a specific gas type, represented as an index.
pub type GasId = usize;

/// Controls the sizes of arrays in the entire simulation.
pub const MAX_NUMBER_OF_GASES: usize = 16;

/// Array that contain a property per [`Gas`] type indexed by [`GasId`]
pub type PerGasArray = f32x16;

/// Returns an iterator over all gas identifiers up to [`MAX_NUMBER_OF_GASES`].
/// These gas identifiers might not map to an existing gas type.
pub fn iter_gas_ids() -> impl Iterator<Item = usize> {
    0..MAX_NUMBER_OF_GASES
}

/// Asserts that a given [`GasId`] is within the valid range.
pub fn assert_gas_id(gas_id: GasId) {
    assert!(gas_id < MAX_NUMBER_OF_GASES);
}

/// Defines the properties of a gas type.
#[derive(Debug, Clone)]
pub struct Gas {
    /// Unique identifier for this gas
    pub gas_id: GasId,
    /// Name for the gas.
    pub name: String,
    /// Molar Heat Capacity for the gas, in Joules per Mole Kelvin
    /// Bigger values require more energy to increase temperature by one Kelvin.
    pub molar_heat_capacity: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_gas_array_size() {
        let f32_size = std::mem::size_of::<f32>();
        let array_size = std::mem::size_of::<PerGasArray>();

        assert_eq!(
            array_size,
            MAX_NUMBER_OF_GASES * f32_size,
            "PerGasArray size does not match MAX_NUMBER_OF_GASES."
        );
    }
}
