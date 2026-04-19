use serde::{Deserialize, Serialize};
use std::{array, fmt};

use crate::{
    GasId, IDEAL_GAS_CONSTANT, MAX_NUMBER_OF_GASES, MINIMUM_AMOUNT_OF_SUBSTANCE, PerGasArray,
    assert_gas_id, gas_list::GasList, mixture_template::TemplatableMixture, per_gas_array,
    prelude::MixtureTemplate,
};

/// Pressure (in Pascals) for each gas type, used for partial pressures.
pub type PressureArray = PerGasArray;
/// Molar Quantity in moles of each gas type.
pub type ContentArray = PerGasArray;
/// Volumeless gas mixture (Contents in moles, Energy in joules).
pub type GasComposition = (ContentArray, f32);

/// Base Gas Container type.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GasMixture {
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

impl GasMixture {
    /// Creates a new mixture with no gases and zero energy.
    pub fn new_empty(volume_m3: f32) -> GasMixture {
        Self::splat(volume_m3, 0.0)
    }

    /// Creates a gas mixture with `amount_moles` of moles for all gases.
    pub fn splat(volume_m3: f32, amount_moles: f32) -> GasMixture {
        if !volume_m3.is_finite() || volume_m3 <= 0.0 {
            panic!("Volume given not finite positive number!");
        }

        let contents = [amount_moles; MAX_NUMBER_OF_GASES];
        let energy = 0.0;

        GasMixture {
            contents,
            energy,
            volume: volume_m3,
        }
    }

    /// Returns the volume of the gas mixture in cubic meters.
    #[inline]
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Returns the sum of all quantities of each gas in moles.
    #[inline]
    pub fn total_moles(&self) -> f32 {
        self.contents.iter().copied().sum()
    }

    /// Computes and returns the heat capacity for the gas in J/K.
    #[inline]
    pub fn heat_capacity(&self, gas_list: &GasList) -> f32 {
        let heat_capacities = gas_list.get_molar_heat_capacities();

        self.contents
            .iter()
            .copied()
            .zip(heat_capacities.iter())
            .map(|(quantity, &c_v)| c_v * quantity)
            .sum()
    }

    /// Computes and returns the temperature of the gas mixture in Kelvin.
    #[inline]
    pub fn temperature(&self, gas_list: &GasList) -> f32 {
        let heat_capacity = self.heat_capacity(gas_list);

        if heat_capacity <= 0.0 {
            return 0.0; // is absolute 0 even physically possible?
        }

        self.energy / heat_capacity
    }

    /// Computes and returns the pressure of the gas mixture in Pascals.
    pub fn pressure(&self, gas_list: &GasList) -> f32 {
        let temperature_k = self.temperature(gas_list);

        ideal_gas_law_pressure(self.total_moles(), temperature_k, self.volume)
    }

    /// Computes and returns the partial pressures of the gas mixture in Pascals.
    pub fn partial_pressure(&self, gas_list: &GasList, gas_id: GasId) -> f32 {
        let gas_moles = self.contents.get(gas_id).copied().unwrap_or(0.0);

        if gas_moles <= 0.0 {
            return 0.0;
        }

        let temperature_k = self.temperature(gas_list);
        ideal_gas_law_pressure(gas_moles, temperature_k, self.volume)
    }

    /// Dalton's Law of partial pressures in Pascals.
    pub fn partial_pressures(&self, gas_list: &GasList) -> PressureArray {
        let temperature_k = self.temperature(gas_list);

        self.contents.map(|moles| {
            // BUG: this does not check for <= 0 moles
            ideal_gas_law_pressure(moles, temperature_k, self.volume)
        })
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

    /// Removes a gas, but does not remove the gas' energy.
    pub fn remove_gas(&mut self, gas_id: GasId) {
        assert_gas_id(gas_id);

        self.contents[gas_id] = 0.0;
    }

    /// Adds a molar quantity of a gas given its [`GasId`]
    pub fn add_gas(&mut self, gas_id: GasId, amount_moles: f32) {
        assert_gas_id(gas_id);

        self.contents[gas_id] = amount_moles;
    }

    /// Sets the internal energy of a mixture in joules
    pub fn set_energy(&mut self, energy_j: f32) {
        self.energy = energy_j;
    }

    /// Sets the internal temperature of a mixture in relation to its contents.
    pub fn set_temperature(&mut self, gas_list: &GasList, temperature_k: f32) {
        let energy_j = temperature_k * self.heat_capacity(gas_list);

        self.set_energy(energy_j);
    }

    /// Computes and returns the partial pressures of each gas in the mixture.
    pub fn delta_pressures(&self, other: &GasMixture, gas_list: &GasList) -> PressureArray {
        let pressure_self = self.partial_pressures(gas_list);
        let pressure_other = other.partial_pressures(gas_list);

        array::from_fn::<f32, MAX_NUMBER_OF_GASES, _>(|idx| {
            pressure_self[idx] - pressure_other[idx]
        })
    }

    /// Equalizes both gas contents and energy in proportion to both volumes.
    pub fn equalize(&mut self, other: &mut GasMixture) {
        // BUG
        let total_volume = self.volume + other.volume;
        let self_factor = self.volume / total_volume;
        let other_factor = other.volume / total_volume;

        let contents = std::array::from_fn(|i| {
            self.contents[i] * self_factor + other.contents[i] * other_factor
        });

        let energy = self.energy * self_factor + other.energy * other_factor;

        self.contents = contents;
        self.energy = energy;
        other.contents = contents;
        other.energy = energy;
    }

    /// Equalizes the temperatures between two mixtures.
    pub fn equalize_temperature(&mut self, other: &mut GasMixture, gas_list: &GasList) {
        let self_heat_capacity = self.heat_capacity(gas_list);
        let other_heat_capacity = other.heat_capacity(gas_list);
        let total_heat_capacity = self_heat_capacity + other_heat_capacity;

        if total_heat_capacity <= 0.0 {
            return;
        }

        let total_energy = self.energy + other.energy;
        let equilibrium_temperature = total_energy / total_heat_capacity;

        self.energy = equilibrium_temperature * self_heat_capacity;
        other.energy = equilibrium_temperature * other_heat_capacity;

        // sanity check
        self.energy = self.energy.max(0.0);
        other.energy = other.energy.max(0.0);
    }

    /// Takes and returns a ratio of the mixture.
    /// It preserves the energy of the mixture.
    pub fn take_ratio(&mut self, ratio: f32) -> GasComposition {
        let ratio = ratio.clamp(0.0, 1.0);

        let mut removed_contents = [0.0; MAX_NUMBER_OF_GASES];

        // FIX: energy removed doesnt account for individual thermal contribution
        let removed_energy = self.energy * ratio;
        self.energy -= removed_energy;

        for (i, amount) in self.contents.iter_mut().enumerate() {
            let removed = *amount * ratio;
            *amount -= removed;

            removed_contents[i] = removed;
        }

        (removed_contents, removed_energy)
    }

    /// Takes and returns a ratio amount given by the volume of the mixture.
    /// It preserves the energy of the mixture.
    pub fn take_volume(&mut self, volume_m3: f32) -> GasComposition {
        if volume_m3 <= 0.0 {
            return ([0.0; MAX_NUMBER_OF_GASES], 0.0);
        }

        let ratio = if volume_m3 >= self.volume {
            1.0
        } else {
            volume_m3 / self.volume
        };

        self.take_ratio(ratio)
    }
}

impl TemplatableMixture for GasMixture {
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
    mixture: &'a GasMixture,
    gas_list: &'a GasList,
}

impl GasMixture {
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn empty_container() {
        let volume_m3 = 1.0;
        let mixture = GasMixture::new_empty(volume_m3);

        mixture.contents.iter().for_each(|moles| {
            assert_eq!(*moles, 0.0);
        });
        assert_eq!(mixture.volume(), volume_m3);
    }
}
