use std::collections::HashMap;

use bevy::prelude::Resource;

use crate::{Gas, GasId, MAX_NUMBER_OF_GASES, PerGasArray, per_gas_array};

/// Molar Heat Capacity in joule per kelvin mole for each gas type.
pub type MolarHeatCapacities = PerGasArray;

/// Serves as a immutable lookup table for defined gases
#[derive(Resource)]
pub struct GasList {
    /// Gas definitions, indexed by their [`crate::GasId`]
    gases: heapless::Vec<Gas, MAX_NUMBER_OF_GASES>,
    /// Cached Molar Heat Capacities at Constant Pressure.
    molar_heat_capacities: MolarHeatCapacities,
    /// Gas name to gas id look up table
    gas_names: HashMap<String, GasId>,
}

impl GasList {
    /// Creates a new Gas List
    /// Returns None if it fails e.g. repeated gas names
    pub fn new(new_gases: Vec<Gas>) -> Self {
        if new_gases.len() > MAX_NUMBER_OF_GASES {
            panic!("Can't initialize atmos with too many gases!");
        }

        let mut gases = heapless::Vec::new();

        let mut molar_heat_capacities = per_gas_array(0.0);
        let mut gas_names = HashMap::<String, GasId>::default();

        for (gas_id, gas) in new_gases.iter().enumerate() {
            gases
                .push(gas.clone())
                .expect("Container should have enough capacity");

            if gas_names.insert(gas.name.clone(), gas_id).is_some() {
                // repeated gas name
                panic!("Repeated gas name!");
            }

            molar_heat_capacities[gas_id] = gas.molar_heat_capacity;
        }

        Self {
            gases,
            molar_heat_capacities,
            gas_names,
        }
    }

    /// Returns true if the list has no defined gases.
    pub fn is_empty(&self) -> bool {
        self.gases.is_empty()
    }

    /// Returns the amount of gas definitions present in the list.
    pub fn len(&self) -> usize {
        self.gases.len()
    }

    /// Retrieves a gas definition indexed by its ID.
    pub fn try_get_gas(&self, gas_id: GasId) -> Option<&Gas> {
        self.gases.get(gas_id)
    }

    /// Iterates over all Gas definitions
    pub fn iter(&self) -> impl Iterator<Item = &Gas> {
        self.gases.iter()
    }

    /// Returns a gas definition by its name.
    pub fn try_get_gas_id_by_name(&self, name: impl Into<String>) -> Option<GasId> {
        self.gas_names.get(&name.into()).copied()
    }

    /// Retrieves the MolarHeatCapacities of each gas.
    pub fn get_molar_heat_capacities(&self) -> &MolarHeatCapacities {
        &self.molar_heat_capacities
    }
}

impl std::fmt::Debug for GasList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("GasList")
            .field("Number of gases", &self.len())
            .field("Gases", &self.gases.as_slice())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn make_gas(name: &str) -> Gas {
        Gas {
            name: name.to_string(),
            molar_heat_capacity: 1.0,
        }
    }

    #[test]
    #[should_panic]
    fn creating_too_many_gases_panics() {
        let too_many = (0..(MAX_NUMBER_OF_GASES + 1))
            .map(|i| make_gas(&format!("g{}", i)))
            .collect::<Vec<_>>();

        let _gas_list = GasList::new(too_many);
    }

    #[test]
    fn getting_a_gas_succeeds() {
        let gases = vec![make_gas("oxygen"), make_gas("nitrogen")];
        let gas_list = GasList::new(gases);

        let maybe = gas_list.try_get_gas(1);

        let g = maybe.expect("gas should exist");

        assert_eq!(g.name, "nitrogen");
    }

    #[test]
    fn getting_a_gas_and_failing_returns_none() {
        let gases = vec![make_gas("helium")];
        let gas_list = GasList::new(gases);
        let none = gas_list.try_get_gas(gas_list.len());

        assert!(none.is_none());
    }

    #[test]
    fn test_len() {
        let size = 5usize;
        let gases = (0..size).map(|i| make_gas(&format!("g{}", i))).collect();
        let gas_list = GasList::new(gases);
        assert_eq!(gas_list.len(), size);
    }
}
