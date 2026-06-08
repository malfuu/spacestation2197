use atmos_primitives::equations::*;
use atmos_primitives::gas_mixture::{HeatCapacityArray, MolarHeatCapacities, PressureArray};
use atmos_primitives::prelude::*;
use wide::f32x16;

const TILE_VOLUME: f32 = 2.5;

pub type TileMoles = ContentArray;
pub type TileEnergy = f32;

#[derive(Default, Clone, Copy, Debug)]
pub struct CachedTile {
    pub temperature: f32,
    pub partial_pressures: PressureArray,
    pub pressure: f32,
    pub heat_capacity: f32,
    pub heat_capacities: HeatCapacityArray,
}

pub struct TileMixtureView<'a> {
    moles: &'a TileMoles,
    energy: &'a TileEnergy,
}

impl<'a> TileMixtureView<'a> {
    pub fn new(moles: &'a TileMoles, energy: &'a TileEnergy) -> Self {
        Self { moles, energy }
    }
}

impl ThermodynamicMixture for TileMixtureView<'_> {
    fn moles(&self) -> &PerGasArray {
        self.moles
    }

    fn energy(&self) -> &f32 {
        self.energy
    }

    fn total_moles(&self) -> f32 {
        mixture_total_moles(self.moles())
    }

    fn partial_heat_capacities(
        &self,
        molar_heat_capacities: &MolarHeatCapacities,
    ) -> HeatCapacityArray {
        mixture_partial_heat_capacities(self.moles(), molar_heat_capacities)
    }

    fn heat_capacity(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32 {
        mixture_heat_capacity(self.moles(), molar_heat_capacities)
    }

    fn temperature(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32 {
        mixture_temperature(*self.energy(), self.heat_capacity(molar_heat_capacities))
    }
}

impl VolumetricMixture for TileMixtureView<'_> {
    fn volume(&self) -> &f32 {
        &TILE_VOLUME
    }

    fn partial_pressures(&self, gas_list: &GasList) -> PressureArray {
        mixture_partial_pressures(
            self.moles(),
            self.temperature(gas_list.get_molar_heat_capacities()),
            *self.volume(),
        )
    }

    fn pressure(&self, gas_list: &GasList) -> f32 {
        mixture_pressure(
            self.total_moles(),
            self.temperature(gas_list.get_molar_heat_capacities()),
            *self.volume(),
        )
    }
}

pub struct TileMixtureViewMut<'a> {
    moles: &'a mut TileMoles,
    energy: &'a mut TileEnergy,
}

impl<'a> TileMixtureViewMut<'a> {
    pub fn new(moles: &'a mut TileMoles, energy: &'a mut TileEnergy) -> Self {
        Self { moles, energy }
    }

    pub fn clear(&mut self) {
        *self.moles = f32x16::ZERO;
        *self.energy = 0.0;
    }

    pub fn cull(&mut self) {
        let min_moles = f32x16::splat(atmos_primitives::MINIMUM_AMOUNT_OF_SUBSTANCE);
        let mask = (*self.moles).simd_ge(min_moles);
        *self.moles = mask.blend(*self.moles, f32x16::ZERO);
    }

    pub fn moles_mut(&mut self) -> &mut ContentArray {
        self.moles
    }

    pub fn energy_mut(&mut self) -> &mut f32 {
        self.energy
    }
}

impl ThermodynamicMixture for TileMixtureViewMut<'_> {
    fn moles(&self) -> &ContentArray {
        self.moles
    }

    fn energy(&self) -> &f32 {
        self.energy
    }

    fn total_moles(&self) -> f32 {
        mixture_total_moles(self.moles())
    }

    fn partial_heat_capacities(
        &self,
        molar_heat_capacities: &MolarHeatCapacities,
    ) -> HeatCapacityArray {
        mixture_partial_heat_capacities(self.moles(), molar_heat_capacities)
    }

    fn heat_capacity(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32 {
        mixture_heat_capacity(self.moles(), molar_heat_capacities)
    }

    fn temperature(&self, molar_heat_capacities: &MolarHeatCapacities) -> f32 {
        mixture_temperature(*self.energy(), self.heat_capacity(molar_heat_capacities))
    }
}

impl VolumetricMixture for TileMixtureViewMut<'_> {
    fn volume(&self) -> &f32 {
        &TILE_VOLUME
    }

    fn partial_pressures(&self, gas_list: &GasList) -> PressureArray {
        mixture_partial_pressures(
            self.moles(),
            self.temperature(gas_list.get_molar_heat_capacities()),
            *self.volume(),
        )
    }

    fn pressure(&self, gas_list: &GasList) -> f32 {
        mixture_pressure(
            self.total_moles(),
            self.temperature(gas_list.get_molar_heat_capacities()),
            *self.volume(),
        )
    }
}

impl TemplatableMixture for TileMixtureViewMut<'_> {
    fn apply_template(&mut self, template: &MixtureTemplate, gas_list: &GasList) {
        self.clear();

        let total_moles =
            ideal_gas_law_moles(template.pressure_pa, *self.volume(), template.temperature_k);

        for (gas_id, &frac) in template.composition.as_array().iter().enumerate() {
            if frac > 0.0 {
                self.moles_mut().as_mut_array()[gas_id] = total_moles * frac;
            }
        }

        *self.energy_mut() =
            template.temperature_k * self.heat_capacity(gas_list.get_molar_heat_capacities());
    }
}
