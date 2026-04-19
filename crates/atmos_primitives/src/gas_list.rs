use std::collections::HashMap;

use bevy::prelude::Resource;
use uom::{ConstZero, si::f32::MolarHeatCapacity};

use crate::{Gas, GasId, MAX_NUMBER_OF_GASES, assert_gas_id};

pub type MolarHeatCapacities = [MolarHeatCapacity; MAX_NUMBER_OF_GASES];

/// Serves as a immutable lookup table for defined gases
#[derive(Resource)]
pub struct GasList {
    gases: heapless::Vec<Gas, MAX_NUMBER_OF_GASES>,
    /// Cached Molar Heat Capacities at Constant Pressure.
    molar_heat_capacities: MolarHeatCapacities,
    /// gas name to gas id look up table
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

        let mut molar_heat_capacities = [MolarHeatCapacity::ZERO; MAX_NUMBER_OF_GASES];
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

    pub fn len(&self) -> usize {
        self.gases.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.gases.is_empty()
    }

    /// Get a Gas from the list
    ///
    /// # Safety
    /// gas_id must be under len()
    pub unsafe fn get_gas(&self, gas_id: GasId) -> &Gas {
        assert_gas_id(gas_id);

        // # Safety
        // correct gas_id given by the caller
        unsafe { self.gases.get_unchecked(gas_id) }
    }

    pub fn try_get_gas(&self, gas_id: GasId) -> Option<&Gas> {
        self.gases.get(gas_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Gas> {
        self.gases.iter()
    }

    pub fn try_get_gas_id_by_name(&self, name: impl Into<String>) -> Option<GasId> {
        self.gas_names.get(&name.into()).copied()
    }

    pub fn get_gas_names(&self) -> impl Iterator<Item = &String> {
        self.gas_names.keys()
    }

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
    use uom::si::molar_heat_capacity::joule_per_kelvin_mole;

    fn make_gas(name: &str) -> Gas {
        Gas {
            name: name.to_string(),
            molar_heat_capacity: MolarHeatCapacity::new::<joule_per_kelvin_mole>(1.0),
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
    fn unsafe_get_gas_succeeds() {
        let gases = vec![make_gas("oxygen")];
        let gas_list = GasList::new(gases);

        let g0 = unsafe { gas_list.get_gas(0) };

        assert_eq!(g0.name, "oxygen");
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

    #[test]
    fn test_empty() {
        let gas_list = GasList::new(vec![]);
        assert!(gas_list.is_empty());
        assert_eq!(gas_list.len(), 0);
        let collected: Vec<&Gas> = gas_list.iter().collect();
        assert!(collected.is_empty());
    }
}
