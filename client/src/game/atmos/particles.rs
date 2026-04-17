//! Particles for displaying environmental atmos mixtures and their effects.
//! This includes both mixture particles and wind particles.

use bevy::prelude::*;

use atmos::engine::chunk::Mixtures;
use atmos::prelude::*;
use bevy_hanabi::{
    AccelModifier, Attribute, ColorOverLifetimeModifier, EffectAsset, EffectMaterial,
    EffectProperties, ExprWriter, HanabiPlugin, OrientMode, OrientModifier, ParticleEffect,
    ParticleTextureModifier, ScalarType, SetAttributeModifier, SpawnerSettings, Value, VectorType,
};
use grid::CHUNK_SIZE;
use shared::defines::TILE_CUBOID;
use uom::si::pressure::kilopascal;

use crate::base::grid::ChunkEntities;

const MIN_PRESSURE_KPA: f32 = 20.0;
const STANDARD_ATMOSPHERE_KPA: f32 = 101.0;

const PARTICLE_PROPERTY_WIND: &str = "wind";
const PARTICLE_PROPERTY_STRENGTH: &str = "strength";

const AIR_PARTICLE_CAPACITY: u32 = 256;
const AIR_PARTICLE_SIZE: f32 = 0.10;
const AIR_PARTICLE_LIFETIME: f32 = 16.0;
const AIR_PARTICLE_ACCELERATION: f32 = 0.0;
const AIR_PARTICLE_VELOCITY: Vec3 = vec3(0.1, 0.0, 0.1);

pub(super) struct ClientAtmosParticlesPlugin;

impl Plugin for ClientAtmosParticlesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(HanabiPlugin)
            .add_systems(Startup, build_air_particles)
            // TODO: currently, spawning an effect for each tile halves FPS
            // a better particle solution is required.
            // .add_systems(Update, update_mixture_particles)
            // .add_observer(on_chunk_entities_add)
        ;
    }
}

#[derive(Resource)]
pub struct AtmosParticleEffects {
    pub effect: Handle<EffectAsset>,
    pub image: Handle<Image>,
}

fn tile_cuboid_spawn_modifier(writer: &ExprWriter) -> SetAttributeModifier {
    let rand_vec = writer.rand(VectorType::VEC3F);
    let cuboid_size = writer.lit(TILE_CUBOID);
    let pos_expr = rand_vec.mul(cuboid_size).expr();
    SetAttributeModifier::new(Attribute::POSITION, pos_expr)
}

fn build_air_particle_effect(
    name: impl Into<String>,
    color: Srgba,
    particle_image_handle: Handle<Image>,
) -> EffectAsset {
    let mut gradient: bevy_hanabi::Gradient<Vec4> = bevy_hanabi::Gradient::<Vec4>::new();
    let full_color = color.to_vec4();
    let transparent_color = full_color * Vec4::new(1., 1., 1., 0.);

    gradient.add_key(0.0, transparent_color);
    gradient.add_key(0.1, full_color);
    gradient.add_key(0.7, full_color);
    gradient.add_key(1.0, transparent_color);

    let writer = ExprWriter::new();
    let init_pos = tile_cuboid_spawn_modifier(&writer);

    // effect strength
    let spawn_chance_prop =
        writer.add_property(PARTICLE_PROPERTY_STRENGTH, Value::Scalar(1.0.into()));
    let spawn_chance_expr = writer.prop(spawn_chance_prop);
    let rand_existing = writer.rand(ScalarType::Float);
    let is_alive_bool = rand_existing.lt(spawn_chance_expr);
    let is_alive_multiplier = is_alive_bool.cast(ScalarType::Float);

    // wind push
    let default_wind = Vec3::ZERO;
    let wind_prop = writer.add_property(PARTICLE_PROPERTY_WIND, Value::Vector(default_wind.into()));
    let wind_expr = writer.prop(wind_prop);

    // fixed size
    let base_size = writer.lit(AIR_PARTICLE_SIZE);
    let size_expr = base_size.expr();
    let init_size = SetAttributeModifier::new(Attribute::SIZE, size_expr);

    // drifting velocity
    let rand_vel = writer.rand(VectorType::VEC3F);
    let vel_offset = writer.lit(vec3(0.5, 1.0, 0.5));
    let vel_scale = writer.lit(AIR_PARTICLE_VELOCITY);
    let vel_expr = rand_vel.sub(vel_offset).mul(vel_scale).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel_expr);

    // lifetime
    let rand_time_multiplier = writer.rand(ScalarType::Float);
    let extra_time = rand_time_multiplier.mul(writer.lit(4.0));
    let base_lifetime = writer.lit(AIR_PARTICLE_LIFETIME).add(extra_time);
    let final_lifetime = base_lifetime.mul(is_alive_multiplier).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, final_lifetime);

    // slight gravity
    let base_accel = writer.lit(vec3(0.0, AIR_PARTICLE_ACCELERATION, 0.0));
    let final_accel = base_accel.add(wind_expr).expr();
    let update_accel = AccelModifier::new(final_accel);

    let texture_slot = writer.lit(0u32).expr();
    let texture_modifier = ParticleTextureModifier::new(texture_slot);

    let mut module = writer.finish();
    module.add_texture_slot("color");

    let orient = OrientModifier::new(OrientMode::FaceCameraPosition);

    EffectAsset::new(
        AIR_PARTICLE_CAPACITY,
        SpawnerSettings::rate(8.0.into()),
        module,
    )
    .with_name(name)
    .init(init_pos)
    .init(init_size)
    .init(init_vel)
    .init(init_lifetime)
    .update(update_accel)
    .render(orient)
    .render(texture_modifier)
    .render(ColorOverLifetimeModifier::new(gradient))
}

fn on_chunk_entities_add(
    add: On<Add, ChunkEntities>,
    mut commands: Commands,
    chunk_entities: Query<&ChunkEntities>,
    atmos_effects: Res<AtmosParticleEffects>,
) {
    let Ok(chunk_grid) = chunk_entities.get(add.entity) else {
        return;
    };

    for &tile_entity in chunk_grid.entities.iter() {
        if tile_entity == Entity::PLACEHOLDER {
            continue;
        }

        let mut properties = bevy_hanabi::EffectProperties::default();
        properties.set(PARTICLE_PROPERTY_STRENGTH, 0.0.into());
        properties.set(PARTICLE_PROPERTY_WIND, Vec3::ZERO.into());

        commands.entity(tile_entity).insert((
            ParticleEffect {
                handle: atmos_effects.effect.clone(),
                prng_seed: Some(rand::random::<u32>()),
            },
            EffectMaterial {
                images: vec![atmos_effects.image.clone()],
            },
            properties,
        ));
    }
}

fn build_air_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: Res<AssetServer>,
) {
    let img = asset_server.load("images/effects/circle_05.png");
    let effect = build_air_particle_effect("plasma", Srgba::WHITE, img.clone());
    let effect_handle = effects.add(effect);

    commands.insert_resource(AtmosParticleEffects {
        effect: effect_handle,
        image: img,
    });
}

pub(super) fn update_mixture_particles(
    gas_list: Res<GasList>,
    mut chunks: Query<(Ref<Mixtures>, &ChunkEntities)>,
    mut particle_query: Query<&mut EffectProperties>,
) {
    for (mixtures, chunk_entities) in chunks {
        if !mixtures.is_changed() {
            continue;
        }

        for x in 0..CHUNK_SIZE as u32 {
            for y in 0..CHUNK_SIZE as u32 {
                let local_pos = UVec2::new(x, y);

                let tile_entity = *chunk_entities
                    .entities
                    .get(local_pos)
                    .expect("Entity grid should be complete.");

                if tile_entity == Entity::PLACEHOLDER {
                    continue;
                }

                let mixture = mixtures.mixtures().get(local_pos).unwrap();
                let pressure_kpa = mixture.pressure(&gas_list).get::<kilopascal>();

                let strength = if pressure_kpa < MIN_PRESSURE_KPA {
                    0.0
                } else {
                    let normalized = (pressure_kpa - MIN_PRESSURE_KPA)
                        / (STANDARD_ATMOSPHERE_KPA - MIN_PRESSURE_KPA);
                    (normalized.powf(1.2) * 0.5).min(1.0)
                };

                if let Ok(mut props) = particle_query.get_mut(tile_entity) {
                    props.set(PARTICLE_PROPERTY_STRENGTH, strength.into());
                }
            }
        }
    }
}
