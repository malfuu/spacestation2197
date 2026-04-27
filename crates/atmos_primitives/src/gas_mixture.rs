//! Gas Mixture and some mathematical definitions.
use crate::{IDEAL_GAS_CONSTANT, PerGasArray, gas_list::GasList};

/// Molar Quantity in moles of each gas type.
pub type ContentArray = PerGasArray;
/// Molar Heat Capacity in joule per kelvin mole for each gas type.
pub type MolarHeatCapacities = PerGasArray;
/// Heat capacity in joule per kelvin in a mixture for each gas type.
pub type HeatCapacityArray = PerGasArray;
/// Partial pressure for each gas type in pascals.
pub type PressureArray = PerGasArray;

/// Volumeless gas mixture (Contents in moles, Energy in joules).
pub type GasComposition = (ContentArray, f32);

/// A data type that implements required to represent a gas made up for multiple partial gases.
pub trait ThermodynamicMixture {
    // TODO: remove gas list as required parameter for all parameters in this trait.
    // as it should be the user's responsibility

    /// Molar quantities of each gas type in the mixture.
    fn moles(&self) -> &ContentArray;

    /// Returns the internal energy of the mixture in joules.
    fn energy(&self) -> &f32;

    /// Returns the sum of all quantities of each gas in moles.
    fn total_moles(&self) -> f32 {
        self.moles().iter().copied().sum()
    }

    /// Returns the partial heat capacity for each gas in J/K.
    fn partial_heat_capacities(
        &self,
        molar_heat_capacities: &MolarHeatCapacities,
    ) -> HeatCapacityArray {
        let moles = self.moles();
        std::array::from_fn(|i| moles[i] * molar_heat_capacities[i])
    }

    /// Returns the heat capacity for the gas in J/K.
    fn heat_capacity(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32 {
        self.partial_heat_capacities(molar_heat_capacities)
            .iter()
            .sum()
    }

    /// Computes and returns the temperature of the gas mixture in Kelvin.
    fn temperature(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32 {
        let heat_capacity = self.heat_capacity(molar_heat_capacities);

        if heat_capacity <= 0.0 {
            return 0.0; // is absolute 0 even physically possible?
        }

        self.energy() / heat_capacity
    }
}

/// A mixture that has properties that require volume to compute.
pub trait VolumetricMixture: ThermodynamicMixture {
    /// Returns the volume of the gas mixture in cubic meters.
    fn volume(&self) -> &f32;

    /// Dalton's Law of partial pressures in Pascals.
    fn partial_pressures(&self, gas_list: &GasList) -> PressureArray {
        let temperature_k = self.temperature(gas_list.get_molar_heat_capacities());

        self.moles().map(|moles| {
            // BUG: this does not check for <= 0 moles
            ideal_gas_law_pressure(moles, temperature_k, *self.volume())
        })
    }

    /// Returns the pressure of the gas mixture in Pascals.
    fn pressure(&self, gas_list: &GasList) -> f32 {
        let temperature_k = self.temperature(gas_list.get_molar_heat_capacities());

        ideal_gas_law_pressure(self.total_moles(), temperature_k, *self.volume())
    }

    /// Computes and returns the difference in partial pressures in pascals
    /// between this mixture and another volumetric mixture.
    fn delta_pressures<T: VolumetricMixture>(
        &self,
        other: &T,
        gas_list: &GasList,
    ) -> PressureArray {
        let pressure_self = self.partial_pressures(gas_list);
        let pressure_other = other.partial_pressures(gas_list);

        std::array::from_fn(|idx| pressure_self[idx] - pressure_other[idx])
    }
}

/// Ideal Gas Law equation to resolve for moles
#[inline]
#[must_use]
pub fn ideal_gas_law_moles(pressure_pa: f32, volume_m3: f32, temperature_k: f32) -> f32 {
    if temperature_k <= 0.0 {
        return 0.0;
    }

    (pressure_pa * volume_m3) / (IDEAL_GAS_CONSTANT * temperature_k)
}

/// Ideal Gas Law equation to resolve for pressure in Pascals
#[inline]
#[must_use]
pub fn ideal_gas_law_pressure(moles: f32, temperature_k: f32, volume_m3: f32) -> f32 {
    if volume_m3 <= 0.0 {
        panic!("zero volume!");
    }

    (moles * IDEAL_GAS_CONSTANT * temperature_k) / volume_m3
}
