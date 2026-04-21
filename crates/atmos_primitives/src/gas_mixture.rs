use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    IDEAL_GAS_CONSTANT, MAX_NUMBER_OF_GASES, MINIMUM_AMOUNT_OF_SUBSTANCE, PerGasArray,
    gas_list::GasList, mixture_template::TemplatableMixture, per_gas_array,
    prelude::MixtureTemplate,
};

/// Pressure (in Pascals) for each gas type, used for partial pressures.
pub type PressureArray = PerGasArray;
/// Molar Quantity in moles of each gas type.
pub type ContentArray = PerGasArray;
/// Volumeless gas mixture (Contents in moles, Energy in joules).
pub type GasComposition = (ContentArray, f32);

/// A data type that implements required to represent a gas made up for multiple partial gases.
pub trait ThermodynamicMixture {
    // TODO: remove gas list as required parameter for all parameters in this trait.
    // as it should be the user's responsibility

    /// Molar quantities of each gas type in the mixture.
    fn moles(&self) -> PerGasArray;
    /// Returns the sum of all quantities of each gas in moles.
    fn total_moles(&self) -> f32;

    /// Returns the internal energy of the mixture in joules.
    fn energy(&self) -> f32;

    /// Sets the internal energy of a mixture in joules
    fn set_energy(&mut self, energy_j: f32);

    /// Returns the partial heat capacity for each gas in J/K.
    fn partial_heat_capacities(&self, gas_list: &GasList) -> PerGasArray;
    /// Returns the heat capacity for the gas in J/K.
    fn heat_capacity(&self, gas_list: &GasList) -> f32;

    /// Computes and returns the temperature of the gas mixture in Kelvin.
    fn temperature(&self, gas_list: &GasList) -> f32;
}

/// A mixture that has properties that require volume to compute.
pub trait VolumetricMixture {
    /// Returns the volume of the gas mixture in cubic meters.
    fn volume(&self) -> f32;

    /// Dalton's Law of partial pressures in Pascals.
    fn partial_pressures(&self, gas_list: &GasList) -> PressureArray;

    /// Returns the pressure of the gas mixture in Pascals.
    fn pressure(&self, gas_list: &GasList) -> f32;

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

/// Base Gas Container type.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BasicGasMixture {
    /// Molar quantities of each gas type in the mixture.
    pub contents: ContentArray,
    /// How much energy in joules this mixture has.
    pub energy: f32,
    /// The volume of the mixture in cubic meters.
    volume: f32,
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

impl BasicGasMixture {
    /// Creates a new mixture with no gases and zero energy.
    pub fn new_empty(volume_m3: f32) -> BasicGasMixture {
        if !volume_m3.is_finite() || volume_m3 <= 0.0 {
            panic!("Volume given not finite positive number!");
        }

        let contents = [0.0; MAX_NUMBER_OF_GASES];
        let energy = 0.0;

        BasicGasMixture {
            contents,
            energy,
            volume: volume_m3,
        }
    }

    /// Culls any molar amounts less then [`MINIMUM_AMOUNT_OF_SUBSTANCE`],
    /// considered meaningless.
    pub fn cull(&mut self) {
        for gas_id in 0..MAX_NUMBER_OF_GASES {
            if self.contents[gas_id] < MINIMUM_AMOUNT_OF_SUBSTANCE {
                self.contents[gas_id] = 0.0;
            }
        }
    }

    /// Clears any contents and removes all internal energy.
    pub fn clear(&mut self) {
        self.contents = per_gas_array(0.0);
        self.energy = 0.0;
    }
}

impl ThermodynamicMixture for BasicGasMixture {
    fn moles(&self) -> PerGasArray {
        self.contents
    }

    fn total_moles(&self) -> f32 {
        self.moles().iter().copied().sum()
    }

    fn energy(&self) -> f32 {
        self.energy
    }

    fn partial_heat_capacities(&self, gas_list: &GasList) -> PerGasArray {
        let molar_caps = gas_list.get_molar_heat_capacities();
        std::array::from_fn(|i| self.contents[i] * molar_caps[i])
    }

    fn heat_capacity(&self, gas_list: &GasList) -> f32 {
        self.partial_heat_capacities(gas_list).iter().sum()
    }

    fn temperature(&self, gas_list: &GasList) -> f32 {
        let heat_capacity = self.heat_capacity(gas_list);

        if heat_capacity <= 0.0 {
            return 0.0; // is absolute 0 even physically possible?
        }

        self.energy / heat_capacity
    }

    fn set_energy(&mut self, energy_j: f32) {
        self.energy = energy_j;
    }
}

impl VolumetricMixture for BasicGasMixture {
    fn volume(&self) -> f32 {
        self.volume
    }

    fn partial_pressures(&self, gas_list: &GasList) -> PressureArray {
        let temperature_k = self.temperature(gas_list);

        self.contents.map(|moles| {
            // BUG: this does not check for <= 0 moles
            ideal_gas_law_pressure(moles, temperature_k, self.volume)
        })
    }

    fn pressure(&self, gas_list: &GasList) -> f32 {
        let temperature_k = self.temperature(gas_list);

        ideal_gas_law_pressure(self.total_moles(), temperature_k, self.volume)
    }
}

impl TemplatableMixture for BasicGasMixture {
    fn apply_template(&mut self, template: &MixtureTemplate, gas_list: &GasList) {
        self.clear();

        let total_moles =
            ideal_gas_law_moles(template.pressure_pa, self.volume(), template.temperature_k);

        for (gas_id, &frac) in template.composition.iter().enumerate() {
            if frac > 0.0 {
                self.contents[gas_id] = total_moles * frac;
            }
        }

        self.energy = template.temperature_k * self.heat_capacity(gas_list);
    }
}

/// Debug wrapper for [`GasMixture`] that accesses the [`GasList`]
pub struct GasMixtureDebugWrapper<'a> {
    mixture: &'a BasicGasMixture,
    gas_list: &'a GasList,
}

impl BasicGasMixture {
    /// Helper method to create the wrapper for debugging.
    pub fn debug_with<'a>(&'a self, gas_list: &'a GasList) -> GasMixtureDebugWrapper<'a> {
        GasMixtureDebugWrapper {
            mixture: self,
            gas_list,
        }
    }
}

impl fmt::Debug for GasMixtureDebugWrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("GasMixture");

        let temperature = self.mixture.temperature(self.gas_list);
        let pressure = self.mixture.pressure(self.gas_list);

        s.field("temperature (k)", &temperature);
        s.field("pressure (pa)", &pressure);

        s.field("volume (m^3)", &self.mixture.volume);
        s.field("energy (j)", &self.mixture.energy);

        let active_contents: Vec<(String, f32)> = self
            .mixture
            .contents
            .iter()
            .enumerate()
            .filter(|&(_, &q)| q > 0.0)
            .map(|(i, &q)| (self.gas_list.try_get_gas(i).unwrap().name.clone(), q))
            .collect();

        s.field("contents_by_id", &active_contents);
        s.finish()
    }
}
