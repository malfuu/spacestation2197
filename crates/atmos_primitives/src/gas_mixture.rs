//! Gas Mixture and some mathematical definitions.
use crate::{PerGasArray, gas_list::GasList};

/// Molar Quantity in Moles of each gas type.
pub type ContentArray = PerGasArray;
/// Molar Heat Capacity in Joules per Kelvin Mole for each gas type.
pub type MolarHeatCapacities = PerGasArray;
/// Heat capacity in Joules per Kelvin in a mixture for each gas type.
pub type HeatCapacityArray = PerGasArray;
/// Partial pressure for each gas type in Pascals.
pub type PressureArray = PerGasArray;

/// Volumeless gas mixture (Contents in Moles, Energy in Joules).
pub type GasComposition = (ContentArray, f32);

/// A data type that implements required to represent a gas made up for multiple partial gases.
pub trait ThermodynamicMixture {
    // TODO: remove gas list as required parameter for all parameters in this trait.
    // as it should be the user's responsibility

    /// Molar quantities of each gas type in the mixture.
    fn moles(&self) -> &ContentArray;

    /// Returns the internal energy of the mixture in Joules.
    fn energy(&self) -> &f32;

    /// Returns the sum of all quantities of each gas in Moles.
    fn total_moles(&self) -> f32;

    /// Returns the partial heat capacity for each gas in Joules per Kelvin.
    fn partial_heat_capacities(
        &self,
        molar_heat_capacities: &MolarHeatCapacities,
    ) -> HeatCapacityArray;

    /// Returns the heat capacity for the gas in Joules per Kelvin.
    fn heat_capacity(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32;

    /// Computes and returns the temperature of the gas mixture in Kelvin.
    fn temperature(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32;
}

/// A mixture that has properties that require volume to compute.
pub trait VolumetricMixture: ThermodynamicMixture {
    /// Returns the volume of the gas mixture in Cubic Meters.
    fn volume(&self) -> &f32;

    /// Dalton's Law of partial pressures in Pascals.
    fn partial_pressures(&self, gas_list: &GasList) -> PressureArray;

    /// Returns the pressure of the gas mixture in Pascals.
    fn pressure(&self, gas_list: &GasList) -> f32;
}
