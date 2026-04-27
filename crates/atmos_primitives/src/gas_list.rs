//! Gas list management and registry storage.
use std::collections::{HashMap, HashSet};

use bevy::prelude::Resource;

use crate::{Gas, GasId, MAX_NUMBER_OF_GASES, gas_mixture::MolarHeatCapacities, per_gas_array};

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
    pub fn new(mut new_gases: Vec<Gas>) -> Self {
        if new_gases.len() > MAX_NUMBER_OF_GASES {
            panic!("Can't initialize atmos with too many gases!");
        }

        new_gases.sort_by_key(|gas| gas.gas_id);

        let mut gases = heapless::Vec::new();
        let mut molar_heat_capacities = per_gas_array(0.0);
        let mut gas_names = HashMap::<String, GasId>::default();
        let mut seen_ids = HashSet::<GasId>::new();

        for gas in new_gases.iter() {
            let gas_id = gas.gas_id;

            if gas_id != gases.len() {
                panic!("Gas IDs must be sequential! (Starting from 0)");
            }

            if !seen_ids.insert(gas_id) {
                panic!("Repeated gas ID!");
            }

            if gas_names.insert(gas.name.clone(), gas_id).is_some() {
                // repeated gas name
                panic!("Repeated gas name!");
            }

            molar_heat_capacities[gas_id] = gas.molar_heat_capacity;
            gases
                .push(gas.clone())
                .expect("Container should have enough capacity");
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

    fn make_gas(gas_id: GasId) -> Gas {
        Gas {
            gas_id: gas_id.clone(),
            name: format!("{gas_id}"),
            molar_heat_capacity: 1.0,
        }
    }

    #[test]
    #[should_panic]
    fn creating_too_many_gases_panics() {
        let too_many = (0..(MAX_NUMBER_OF_GASES + 1))
            .map(|i| make_gas(i))
            .collect::<Vec<_>>();

        let _gas_list = GasList::new(too_many);
    }

    #[test]
    fn getting_a_gas_succeeds() {
        let gases = vec![make_gas(0), make_gas(1)];
        let gas_list = GasList::new(gases);

        let maybe = gas_list.try_get_gas(1);

        let g = maybe.expect("gas should exist");

        assert_eq!(g.name, "1");
    }

    #[test]
    fn getting_a_gas_and_failing_returns_none() {
        let gases = vec![make_gas(0)];
        let gas_list = GasList::new(gases);
        let none = gas_list.try_get_gas(gas_list.len());

        assert!(none.is_none());
    }

    #[test]
    fn test_len() {
        let size = 5usize;
        let gases = (0..size).map(|i| make_gas(i)).collect();
        let gas_list = GasList::new(gases);
        assert_eq!(gas_list.len(), size);
    }

    #[test]
    fn test_is_empty() {
        let gas_list = GasList::new(vec![]);
        assert!(gas_list.is_empty());
        assert_eq!(gas_list.len(), 0);
    }

    #[test]
    fn sorting_out_of_order_gases_succeeds() {
        let gases = vec![make_gas(1), make_gas(0), make_gas(2)];
        let gas_list = GasList::new(gases);

        assert_eq!(gas_list.try_get_gas(0).unwrap().name, "0");
        assert_eq!(gas_list.try_get_gas(2).unwrap().name, "2");
    }

    #[test]
    fn getting_gas_id_by_name_succeeds() {
        let gases = vec![make_gas(0), make_gas(1)];
        let gas_list = GasList::new(gases);

        let id = gas_list.try_get_gas_id_by_name("1");
        assert_eq!(id, Some(1));
    }

    #[test]
    fn getting_gas_id_by_unknown_name_returns_none() {
        let gas_list = GasList::new(vec![make_gas(0)]);
        let none = gas_list.try_get_gas_id_by_name("unknown_gas");
        assert!(none.is_none());
    }

    #[test]
    #[should_panic]
    fn missing_sequential_ids_panics() {
        let gases = vec![make_gas(0), make_gas(2)];
        let _gas_list = GasList::new(gases);
    }

    #[test]
    #[should_panic]
    fn starting_with_non_zero_id_panics() {
        let gases = vec![make_gas(1), make_gas(2)];
        let _gas_list = GasList::new(gases);
    }

    #[test]
    #[should_panic]
    fn repeated_gas_ids_panics() {
        let gases = vec![make_gas(0), make_gas(0)];
        let _gas_list = GasList::new(gases);
    }

    #[test]
    #[should_panic]
    fn repeated_gas_names_panics() {
        let mut gas1 = make_gas(0);
        let mut gas2 = make_gas(1);

        gas1.name = "Oxygen".to_string();
        gas2.name = "Oxygen".to_string();

        let _gas_list = GasList::new(vec![gas1, gas2]);
    }

    #[test]
    fn molar_heat_capacities_are_stored_correctly() {
        let mut gas0 = make_gas(0);
        gas0.molar_heat_capacity = 20.0;

        let mut gas1 = make_gas(1);
        gas1.molar_heat_capacity = 30.0;

        let gas_list = GasList::new(vec![gas0, gas1]);
        let capacities = gas_list.get_molar_heat_capacities();

        assert_eq!(capacities[0], 20.0);
        assert_eq!(capacities[1], 30.0);
    }
}
