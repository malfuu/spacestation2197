use serde::{Deserialize, Serialize};
use std::{array, fmt};
use uom::{
    ConstZero,
    si::{
        amount_of_substance::mole, f32::*, pressure::pascal, thermodynamic_temperature::kelvin,
        volume::cubic_meter,
    },
};

use crate::{
    GasId, IDEAL_GAS_CONSTANT, MAX_NUMBER_OF_GASES, MINIMUM_AMOUNT_OF_SUBSTANCE, assert_gas_id,
    gas_list::GasList,
};

/// Pressure for each gas type, used for partial pressures.
pub type PressureArray = [Pressure; MAX_NUMBER_OF_GASES];
/// Molar Quantity of each gas type.
pub type ContentArray = [AmountOfSubstance; MAX_NUMBER_OF_GASES];
/// Volumeless gas mixture.
pub type GasComposition = (ContentArray, Energy);

/// Base Gas Container type.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GasMixture {
    /// Molar quantities of each gas type in the mixture.
    pub contents: ContentArray,
    /// How much energy in joules this mixture has.
    pub energy: Energy,
    /// The volume of the mixture.
    volume: Volume,
}

/// Ideal Gas Law equation to resolve for moles
#[inline]
#[must_use]
pub fn ideal_gas_law_moles(
    pressure: Pressure,
    volume: Volume,
    temperature: ThermodynamicTemperature,
) -> AmountOfSubstance {
    if temperature <= ThermodynamicTemperature::ZERO {
        return AmountOfSubstance::ZERO;
    }

    AmountOfSubstance::new::<mole>(
        (pressure.get::<pascal>() * volume.get::<cubic_meter>())
            / (IDEAL_GAS_CONSTANT * temperature.get::<kelvin>()),
    )
}

/// Ideal Gas Law equation to resolve for pressure
#[inline]
#[must_use]
pub fn ideal_gas_law_pressure(
    moles: AmountOfSubstance,
    temperature: ThermodynamicTemperature,
    volume: Volume,
) -> Pressure {
    if volume <= Volume::ZERO {
        panic!("zero volume!");
    }

    Pressure::new::<pascal>(
        moles.get::<mole>() * IDEAL_GAS_CONSTANT * temperature.get::<kelvin>()
            / volume.get::<cubic_meter>(),
    )
}

impl GasMixture {
    /// Creates a new mixture with no gases and zero energy.
    pub fn new_empty(volume: Volume) -> GasMixture {
        Self::splat(volume, AmountOfSubstance::ZERO)
    }

    /// Creates a gas mixture with `amount` of moles for all gases.
    pub fn splat(volume: Volume, amount: AmountOfSubstance) -> GasMixture {
        if !volume.is_finite() || volume <= Volume::ZERO {
            panic!("Volume given not finite positive number!");
        }

        let contents = [amount; MAX_NUMBER_OF_GASES];
        let energy = Energy::ZERO;

        GasMixture {
            contents,
            energy,
            volume,
        }
    }

    /// Returns the volume of the gas mixture.
    #[inline]
    pub fn volume(&self) -> Volume {
        self.volume
    }

    /// Returns the sum of all quantities of each gas.
    #[inline]
    pub fn total_moles(&self) -> AmountOfSubstance {
        self.contents.iter().copied().sum()
    }

    /// Computes and returns the heat capacity for the gas.
    #[inline]
    pub fn heat_capacity(&self, gas_list: &GasList) -> HeatCapacity {
        let heat_capacities = gas_list.get_molar_heat_capacities();

        self.contents
            .iter()
            .copied()
            .zip(heat_capacities.iter())
            .map(|(quantity, &c_v)| c_v * quantity)
            .sum()
    }

    /// Computes and returns the temperature of the gas mixture.
    #[inline]
    pub fn temperature(&self, gas_list: &GasList) -> ThermodynamicTemperature {
        let heat_capacity = self.heat_capacity(gas_list);

        if heat_capacity <= HeatCapacity::ZERO {
            return ThermodynamicTemperature::ZERO; // is absolute 0 even physically possible?
        }

        // ThermodynamicTemperature::new::<kelvin>((self.energy / heat_capacity).value)
        ThermodynamicTemperature::new::<kelvin>((self.energy / heat_capacity).value)
    }

    /// Computes and returns the pressure of the gas mixture.
    pub fn pressure(&self, gas_list: &GasList) -> Pressure {
        let temperature = self.temperature(gas_list);

        ideal_gas_law_pressure(self.total_moles(), temperature, self.volume)
    }

    /// Computes and returns the partial pressures of the gas mixture.
    pub fn partial_pressure(&self, gas_list: &GasList, gas_id: GasId) -> Pressure {
        let gas_moles = self
            .contents
            .get(gas_id)
            .copied()
            .unwrap_or(AmountOfSubstance::ZERO);

        if gas_moles <= AmountOfSubstance::ZERO {
            return Pressure::ZERO;
        }

        let temperature = self.temperature(gas_list);
        ideal_gas_law_pressure(gas_moles, temperature, self.volume)
    }

    /// Dalton's Law of partial pressures.
    pub fn partial_pressures(&self, gas_list: &GasList) -> PressureArray {
        let temperature = self.temperature(gas_list);

        self.contents.map(|moles| {
            // BUG: this does not check for <= 0 moles
            ideal_gas_law_pressure(moles, temperature, self.volume)
        })
    }

    /// Culls any molar amounts less then [`MINIMUM_AMOUNT_OF_SUBSTANCE`],
    /// considered meaningless.
    pub fn cull(&mut self) {
        let minimum_amount = AmountOfSubstance::new::<mole>(MINIMUM_AMOUNT_OF_SUBSTANCE);

        for gas_id in 0..MAX_NUMBER_OF_GASES {
            if self.contents[gas_id] < minimum_amount {
                self.contents[gas_id] = AmountOfSubstance::ZERO;
            }
        }
    }

    /// Clears any contents and removes all internal energy.
    pub fn clear(&mut self) {
        self.contents = [AmountOfSubstance::ZERO; MAX_NUMBER_OF_GASES];
        self.energy = Energy::ZERO;
    }

    /// Removes a gas, but does not remove the gas' energy.
    pub fn remove_gas(&mut self, gas_id: GasId) {
        assert_gas_id(gas_id);

        self.contents[gas_id] = AmountOfSubstance::ZERO;
    }

    /// Adds a molar quantity of a gas given its [`GasId`]
    pub fn add_gas(&mut self, gas_id: GasId, amount: AmountOfSubstance) {
        assert_gas_id(gas_id);

        self.contents[gas_id] = amount;
    }

    /// Sets the internal energy of a mixture
    pub fn set_energy(&mut self, energy: Energy) {
        self.energy = energy;
    }

    /// Sets the internal temperature of a mixture in relation to it's contents.
    pub fn set_temperature(&mut self, gas_list: &GasList, temperature: ThermodynamicTemperature) {
        let energy = temperature * self.heat_capacity(gas_list);

        self.set_energy(energy);
    }

    /// Computes and returns the partial pressures of each gas in the mixture.
    pub fn delta_pressures(&self, other: &GasMixture, gas_list: &GasList) -> PressureArray {
        let pressure_self = self.partial_pressures(gas_list);
        let pressure_other = other.partial_pressures(gas_list);

        array::from_fn::<Pressure, MAX_NUMBER_OF_GASES, _>(|idx| {
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

        if total_heat_capacity <= HeatCapacity::ZERO {
            return;
        }

        let total_energy = self.energy + other.energy;

        let equilibrium_temperature = total_energy / total_heat_capacity;

        self.energy = equilibrium_temperature * self_heat_capacity;
        other.energy = equilibrium_temperature * other_heat_capacity;

        // sanity check
        self.energy = self.energy.max(Energy::ZERO);
        other.energy = other.energy.max(Energy::ZERO);
    }

    /// Takes and returns a ratio of the mixture.
    /// It preserves the energy of the mixture.
    pub fn take_ratio(&mut self, ratio: f32) -> GasComposition {
        let ratio = ratio.clamp(0.0, 1.0);

        let mut removed_contents = [AmountOfSubstance::ZERO; MAX_NUMBER_OF_GASES];

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
    pub fn take_volume(&mut self, volume: Volume) -> GasComposition {
        if volume <= Volume::ZERO {
            return ([AmountOfSubstance::ZERO; MAX_NUMBER_OF_GASES], Energy::ZERO);
        }

        let ratio = if volume >= self.volume {
            1.0
        } else {
            volume.get::<cubic_meter>() / self.volume.get::<cubic_meter>()
        };

        self.take_ratio(ratio)
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

        s.field("temperature", &temperature);
        s.field("pressure", &pressure);

        s.field("volume", &self.mixture.volume);
        s.field("energy", &self.mixture.energy);

        let active_contents: Vec<(String, AmountOfSubstance)> = self
            .mixture
            .contents
            .iter()
            .enumerate()
            .filter(|&(_, &q)| q.get::<mole>() > 0.0)
            .map(|(i, &q)| (self.gas_list.try_get_gas(i).unwrap().name.clone(), q))
            .collect();

        s.field("contents_by_id", &active_contents);
        s.finish()
    }
}

#[cfg(test)]
mod tests {
    use uom::{
        ConstZero,
        si::{f32::*, volume::liter},
    };

    use crate::prelude::*;

    #[test]
    fn empty_container() {
        let volume = Volume::new::<liter>(1.0);
        let mixture = GasMixture::new_empty(volume);

        mixture.contents.iter().for_each(|moles| {
            assert_eq!(*moles, AmountOfSubstance::ZERO);
        });
        assert_eq!(mixture.volume, volume);
    }
}
