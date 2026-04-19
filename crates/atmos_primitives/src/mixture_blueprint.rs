use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    GasId, MAX_NUMBER_OF_GASES,
    gas_list::GasList,
    gas_mixture::{GasMixture, ideal_gas_law_moles},
};

/// Normalized fractions for each gas type.
pub type FractionArray = [f32; MAX_NUMBER_OF_GASES];

/// Predefined mixture registry.
#[derive(Resource)]
pub struct MixtureList {
    /// Hashmap of mixtures indexed by their name.
    pub list: HashMap<String, MixtureBlueprint>,
}

impl Default for MixtureList {
    fn default() -> Self {
        Self::new()
    }
}

impl MixtureList {
    /// Creates a new [`MixtureList`]
    pub fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }

    /// Adds a new blueprint to the list.
    /// If a mixture with the name already existed, it will return the old one.
    pub fn add(&mut self, blueprint: MixtureBlueprint) -> Option<MixtureBlueprint> {
        self.list.insert(blueprint.name.clone(), blueprint)
    }

    /// Gets a mixture blueprint by its name.
    pub fn get(&self, name: &str) -> Option<&MixtureBlueprint> {
        self.list.get(name)
    }
}

/// Blueprint for a mixture.
pub struct MixtureBlueprint {
    /// Identifying name
    pub name: String,
    /// Target pressure of the gas mixture in Pascals
    pub pressure_pa: f32,
    /// Target temperature of the gas mixture in Kelvin
    pub temperature_k: f32,
    /// Normalized mole fractions
    pub composition: FractionArray,
}

impl MixtureBlueprint {
    /// Creates a new [`MixtureBlueprint`].
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

        let pressure_pa = pressure_kpa * 1000.0;

        let sum: f32 = composition.iter().sum();
        let normalized: FractionArray = if sum > 0.0 {
            composition.map(|f| f / sum)
        } else {
            composition
        };

        Self {
            name: name.into(),
            pressure_pa,
            temperature_k,
            composition: normalized,
        }
    }

    /// Applies a blueprint to a gas mixture.
    pub fn apply_to(&self, mixture: &mut GasMixture, gas_list: &GasList) {
        mixture.clear();

        let total_moles =
            ideal_gas_law_moles(self.pressure_pa, mixture.volume(), self.temperature_k);

        for (gas_id, &frac) in self.composition.iter().enumerate() {
            if frac > 0.0 {
                mixture.contents[gas_id] = total_moles * frac;
            }
        }

        mixture.energy = self.temperature_k * mixture.heat_capacity(gas_list);
    }
}
