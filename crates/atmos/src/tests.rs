use uom::si::{f32::*, molar_heat_capacity::joule_per_kelvin_mole};

use crate::prelude::*;

pub(crate) fn create_generic_gas_list() -> GasList {
    GasList::new(vec![
        Gas {
            name: "oxygen".to_string(),
            molar_heat_capacity: MolarHeatCapacity::new::<joule_per_kelvin_mole>(21.1),
        },
        Gas {
            name: "nitrogen".to_string(),
            molar_heat_capacity: MolarHeatCapacity::new::<joule_per_kelvin_mole>(20.7),
        },
        Gas {
            name: "carbon_dioxide".to_string(),
            molar_heat_capacity: MolarHeatCapacity::new::<joule_per_kelvin_mole>(28.4),
        },
        Gas {
            name: "nitrogen_dioxide".to_string(),
            molar_heat_capacity: MolarHeatCapacity::new::<joule_per_kelvin_mole>(30.0),
        },
        Gas {
            name: "plasma".to_string(),
            molar_heat_capacity: MolarHeatCapacity::new::<joule_per_kelvin_mole>(300.0),
        },
    ])
}
