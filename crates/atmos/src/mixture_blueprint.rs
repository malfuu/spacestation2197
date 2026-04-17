use std::collections::HashMap;

use bevy::prelude::*;
use uom::si::{f32::*, pressure::kilopascal, thermodynamic_temperature::kelvin};

use crate::{
    GasId, MAX_NUMBER_OF_GASES,
    gas_list::GasList,
    gas_mixture::{GasMixture, ideal_gas_law_moles},
};

pub type FractionArray = [f32; MAX_NUMBER_OF_GASES];

#[derive(Resource)]
pub struct MixtureList {
    pub list: HashMap<String, MixtureBlueprint>,
}

impl Default for MixtureList {
    fn default() -> Self {
        Self::new()
    }
}

impl MixtureList {
    pub fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }

    pub fn add(&mut self, blueprint: MixtureBlueprint) -> Option<MixtureBlueprint> {
        self.list.insert(blueprint.name.clone(), blueprint)
    }

    pub fn get(&self, name: &str) -> Option<&MixtureBlueprint> {
        self.list.get(name)
    }
}

pub struct MixtureBlueprint {
    /// Identifying name
    pub name: String,
    /// Target pressure of the gas mixture
    pub pressure: Pressure,
    /// Target temperature of the gas mixture
    pub temperature: ThermodynamicTemperature,
    // Normalized mole fractions
    pub composition: FractionArray,
}

impl MixtureBlueprint {
    pub fn new(
        name: impl Into<String>,
        pressure_kpa: f32,
        temperature_k: f32,
        fractions: Vec<(GasId, f32)>,
    ) -> Self {
        let mut composition: FractionArray = [0.0; MAX_NUMBER_OF_GASES];

        for (id, value) in fractions {
            composition[id] = value;
        }

        let pressure = Pressure::new::<kilopascal>(pressure_kpa);
        let temperature = ThermodynamicTemperature::new::<kelvin>(temperature_k);

        let sum: f32 = composition.iter().sum();
        let normalized: FractionArray = if sum > 0.0 {
            composition.map(|f| f / sum)
        } else {
            composition
        };

        Self {
            name: name.into(),
            pressure,
            temperature,
            composition: normalized,
        }
    }

    pub fn create_mixture(&self, volume: &Volume, gas_list: &GasList) {
        let mut mixture = GasMixture::new_empty(*volume);

        self.apply_to(&mut mixture, gas_list);
    }

    pub fn apply_to(&self, mixture: &mut GasMixture, gas_list: &GasList) {
        mixture.clear();

        let total_moles = ideal_gas_law_moles(self.pressure, mixture.volume(), self.temperature);

        for (gas_id, &frac) in self.composition.iter().enumerate() {
            if frac > 0.0 {
                mixture.contents[gas_id] = total_moles * frac;
            }
        }

        mixture.energy = self.temperature * mixture.heat_capacity(gas_list);
    }
}

#[cfg(test)]
mod tests {
    use uom::si::volume::cubic_meter;

    use super::*;
    use crate::tests::create_generic_gas_list;

    #[test]
    fn blueprint_mixture() {
        let gas_list = create_generic_gas_list();

        let blueprint =
            MixtureBlueprint::new("foo", 101.325, 21., vec![(0, 21.0), (1, 78.0), (2, 1.0)]);

        let mut mixture = GasMixture::new_empty(Volume::new::<cubic_meter>(2.5));

        blueprint.apply_to(&mut mixture, &gas_list);
    }

    #[test]
    fn blueprint_with_invalid_gas_id() {}
}
