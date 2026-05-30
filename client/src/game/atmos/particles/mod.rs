use std::f32::consts::TAU;

/// Collection of particle effects for atmospherics
/// Including fire and mixture particles.
/// Each tile has its own particle effect, which might be inefficient.
/// (an effect per-chunk cant be done, with each pixel being a tile property,
/// until texture sampling is available in `init` in hanabi.)
/// Considering hanabi's lack of batching AND per-effect property setting.
/// This slowness is particularly noticeable in non-release binaries.
/// so I advise you to use `no-atmos-particles` flag to disable particles altogether.
/// Also wind particles do not exist for now.
use bevy::prelude::*;
use bevy_hanabi::*;

use shared::defines::TILE_CUBOID;

const MIXTURE_PARTICLE_NAME: &str = "air_mixture";
const MIXTURE_PARTICLE_CAPACITY: u32 = 32;
const MIXTURE_PARTICLE_LIFETIME: f32 = 8.0;
const MIXTURE_PARTICLE_RATE: f32 = 4.0;
const MIXTURE_PARTICLE_SIZE: f32 = 0.02;
const MIXTURE_PARTICLE_PROPERTY_COLOR: &str = "color";

pub(super) struct AtmosParticlesPlugin;

impl Plugin for AtmosParticlesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(HanabiPlugin)
            .add_systems(Startup, build_particle_effects)
        .add_systems(PostStartup, spawn_effect);
    }
}

fn spawn_effect(
    mut commands: Commands,
    atmos_effects: Res<ParticleEffects >,
) {
    let mut properties = bevy_hanabi::EffectProperties::default();

    commands.spawn((
        Transform::IDENTITY,
        ParticleEffect {
            handle: atmos_effects.mixture_effect.clone(),
            prng_seed: Some(rand::random::<u32>()),
        },
        EffectMaterial {
            images: vec![atmos_effects.particle_image.clone()],
        },
        properties,
    ));
}

#[derive(Resource)]
struct ParticleEffects {
    mixture_effect: Handle<EffectAsset>,
    // fire_effect: Handle<EffectAsset>,
    /// image for particles in both effects
    particle_image: Handle<Image>,
}

fn tile_cuboid_spawn_modifier(writer: &ExprWriter) -> SetAttributeModifier {
    let rand_vec = writer.rand(VectorType::VEC3F);
    let cuboid_size = writer.lit(TILE_CUBOID);
    let pos_expr = rand_vec.mul(cuboid_size).expr();
    SetAttributeModifier::new(Attribute::POSITION, pos_expr)
}

fn random_rotation_modifier(writer: &ExprWriter) -> SetAttributeModifier {
    let rand_angle = writer.rand(ScalarType::Float).mul(writer.lit(TAU)).expr();

    SetAttributeModifier::new(Attribute::F32_0, rand_angle)
}

fn build_particle_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let particle_image = asset_server.load("images/puffs/whitePuff02.png");

    let mixture_asset = build_mixture_effect();
    // let fire_asset = build_fire_effect();

    let mixture_effect = effects.add(mixture_asset);
    // let fire_effect = effects.add(fire_asset);

    commands.insert_resource(ParticleEffects {
        mixture_effect,
        // fire_effect,
        particle_image,
    });
}

fn build_mixture_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    // PROPERTIES
    let color_prop = writer.add_property(MIXTURE_PARTICLE_PROPERTY_COLOR, Vec4::ONE.into());

    // LIFETIME
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME, 
        writer.lit(MIXTURE_PARTICLE_LIFETIME).expr()
    );

    // POSITION
    let init_position = tile_cuboid_spawn_modifier(&writer);
    let init_rotation = random_rotation_modifier(&writer);

    let mut render_orient = OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: Some(writer.attr(Attribute::F32_0).expr()),
    };

    let init_size =
        SetAttributeModifier::new(Attribute::SIZE, writer.lit(MIXTURE_PARTICLE_SIZE).expr());

    // COLOR
    let render_texture = ParticleTextureModifier::new(writer.lit(0u32).expr());

    let mut module = writer.finish();
    module.add_texture_slot("color");

    EffectAsset::new(
        MIXTURE_PARTICLE_CAPACITY,
        SpawnerSettings::rate(MIXTURE_PARTICLE_RATE.into()),
        module,
    )
    .with_name(MIXTURE_PARTICLE_NAME)
    .with_alpha_mode(bevy_hanabi::AlphaMode::Add)
    .init(init_lifetime)
    .init(init_position)
    .init(init_rotation)
    .init(init_size)
    .render(render_orient)
    .render(render_texture)
}

// fn build_fire_effect(
// ) -> EffectAsset {
//     todo!()
// }
