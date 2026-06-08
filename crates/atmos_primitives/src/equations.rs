//! Free functions for thermodynamics and gas mixture equations.
use crate::IDEAL_GAS_CONSTANT;
use crate::gas_mixture::{ContentArray, HeatCapacityArray, MolarHeatCapacities, PressureArray};
use wide::f32x16;

/// Returns the sum of all quantities of each gas in Moles.
#[inline]
pub fn mixture_total_moles(moles: &ContentArray) -> f32 {
    moles.reduce_add()
}

/// Returns the partial heat capacity for each gas in Joules per Kelvin.
#[inline]
pub fn mixture_partial_heat_capacities(
    moles: &ContentArray,
    molar_heat_capacities: &MolarHeatCapacities,
) -> HeatCapacityArray {
    *moles * *molar_heat_capacities
}

/// Returns the heat capacity for the gas in Joules per Kelvin.
#[inline]
pub fn mixture_heat_capacity(
    moles: &ContentArray,
    molar_heat_capacities: &MolarHeatCapacities,
) -> f32 {
    (*moles * *molar_heat_capacities).reduce_add()
}

/// Computes and returns the temperature of the gas mixture in Kelvin.
#[inline]
pub fn mixture_temperature(energy: f32, heat_capacity: f32) -> f32 {
    if heat_capacity <= 0.0 {
        return 0.0;
    }
    energy / heat_capacity
}

/// Dalton's Law of partial pressures in Pascals.
#[inline]
pub fn mixture_partial_pressures(
    moles: &ContentArray,
    temperature_k: f32,
    volume_m3: f32,
) -> PressureArray {
    ideal_gas_law_pressure_wide(*moles, temperature_k, volume_m3)
}

/// Returns the pressure of the gas mixture in Pascals.
#[inline]
pub fn mixture_pressure(total_moles: f32, temperature_k: f32, volume_m3: f32) -> f32 {
    ideal_gas_law_pressure(total_moles, temperature_k, volume_m3)
}

/// Ideal Gas Law equation to resolve for Moles
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

/// SIMD version of Ideal Gas Law equation to resolve for Moles
#[inline]
#[must_use]
pub fn ideal_gas_law_moles_wide(
    pressure_pa: f32x16,
    volume_m3: f32,
    temperature_k: f32x16,
) -> f32x16 {
    let valid_temp = temperature_k.simd_gt(f32x16::ZERO);

    let moles_per_pascal = volume_m3 / (IDEAL_GAS_CONSTANT * temperature_k);
    let moles = pressure_pa * moles_per_pascal;

    valid_temp.blend(moles, f32x16::ZERO)
}

/// SIMD version of Ideal Gas Law equation to resolve for pressure in Pascals
#[inline]
#[must_use]
pub fn ideal_gas_law_pressure_wide(moles: f32x16, temperature_k: f32, volume_m3: f32) -> f32x16 {
    let pascals_per_mole = (IDEAL_GAS_CONSTANT * temperature_k) / volume_m3;
    moles * pascals_per_mole
}
