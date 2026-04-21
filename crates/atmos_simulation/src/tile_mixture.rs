use atmos_primitives::prelude::*;

const TILE_VOLUME: f32 = 2.5;

pub type TileMoles = ContentArray;
pub type TileEnergy = f32;

pub struct TileMixtureView<'a> {
    moles: &'a TileMoles,
    energy: &'a TileEnergy,
}

impl<'a> TileMixtureView<'a> {
    pub fn new(moles: &'a TileMoles, energy: &'a TileEnergy) -> Self {
        Self { moles, energy }
    }

    pub fn contents(&self) -> &'a ContentArray {
        self.moles
    }

    pub fn energy(&self) -> f32 {
        *self.energy
    }
}

impl ThermodynamicMixture for TileMixtureView<'_> {
    fn moles(&self) -> &PerGasArray {
        self.moles
    }

    fn energy(&self) -> &f32 {
        self.energy
    }
}

impl VolumetricMixture for TileMixtureView<'_> {
    fn volume(&self) -> &f32 {
        &TILE_VOLUME
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

    pub fn as_view(&self) -> TileMixtureView<'_> {
        TileMixtureView {
            moles: self.moles,
            energy: self.energy,
        }
    }

    pub fn clear(&mut self) {
        *self.moles = per_gas_array(0.0);
        *self.energy = 0.0;
    }

    pub fn cull(&mut self) {
        for moles in self.moles_mut().iter_mut() {
            if *moles < atmos_primitives::MINIMUM_AMOUNT_OF_SUBSTANCE {
                *moles = 0.0;
            }
        }
    }

    pub fn moles_mut(&mut self) -> &mut PerGasArray {
        self.moles
    }

    pub fn energy_mut(&mut self) -> &mut f32 {
        self.energy
    }
}

impl ThermodynamicMixture for TileMixtureViewMut<'_> {
    fn moles(&self) -> &PerGasArray {
        self.moles
    }

    fn energy(&self) -> &f32 {
        self.energy
    }
}

impl VolumetricMixture for TileMixtureViewMut<'_> {
    fn volume(&self) -> &f32 {
        &TILE_VOLUME
    }
}

impl TemplatableMixture for TileMixtureViewMut<'_> {
    fn apply_template(&mut self, template: &MixtureTemplate, gas_list: &GasList) {
        self.clear();

        let total_moles = atmos_primitives::gas_mixture::ideal_gas_law_moles(
            template.pressure_pa,
            *self.volume(),
            template.temperature_k,
        );

        for (gas_id, &frac) in template.composition.iter().enumerate() {
            if frac > 0.0 {
                self.moles_mut()[gas_id] = total_moles * frac;
            }
        }

        *self.energy_mut() = template.temperature_k * self.heat_capacity(gas_list);
    }
}
