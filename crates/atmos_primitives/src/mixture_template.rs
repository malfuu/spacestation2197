use std::collections::HashMap;

use bevy::prelude::*;

use crate::{GasId, MAX_NUMBER_OF_GASES, PerGasArray, gas_list::GasList};

/// Normalized fractions for each gas type.
pub type FractionArray = PerGasArray;

/// Predefined mixture registry.
#[derive(Resource)]
pub struct MixtureTemplateList {
    /// Hashmap of mixtures indexed by their name.
    pub list: HashMap<String, MixtureTemplate>,
}

impl Default for MixtureTemplateList {
    fn default() -> Self {
        Self::new()
    }
}

impl MixtureTemplateList {
    /// Creates a new [`MixtureList`]
    pub fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }

    /// Adds a new template to the list.
    /// If a mixture with the name already existed, it will return the old one.
    pub fn add(&mut self, template: MixtureTemplate) -> Option<MixtureTemplate> {
        self.list.insert(template.name.clone(), template)
    }

    /// Gets a mixture template by its name.
    pub fn get(&self, name: &str) -> Option<&MixtureTemplate> {
        self.list.get(name)
    }
}

/// Template for a mixture.
pub struct MixtureTemplate {
    /// Identifying name
    pub name: String,
    /// Target pressure of the gas mixture in Pascals
    pub pressure_pa: f32,
    /// Target temperature of the gas mixture in Kelvin
    pub temperature_k: f32,
    /// Normalized mole fractions
    pub composition: FractionArray,
}

impl MixtureTemplate {
    /// Creates a new [`MixtureTemplate`].
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
}

/// A gas mixture that can have [`MixtureTemplate`] applied to it.
pub trait TemplatableMixture {
    /// Applies the [`MixtureTemplate`] into the mixture.
    fn apply_template(&mut self, template: &MixtureTemplate, gas_list: &GasList);
}
