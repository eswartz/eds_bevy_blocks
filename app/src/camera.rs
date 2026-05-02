
#[cfg(feature = "solari")]
use bevy::camera::CameraMainTextureUsages;
use bevy::render::experimental::occlusion_culling::OcclusionCulling;
use bevy_seedling::prelude::*;

use bevy::camera::Exposure;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::oit::OrderIndependentTransparencySettings;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapter;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::Hdr;

use eds_bevy_common::*;

#[cfg(feature = "solari")]
use bevy::anti_alias::dlss::{
    Dlss, DlssProjectId, DlssRayReconstructionFeature, DlssRayReconstructionSupported,
};
#[cfg(feature = "solari")]
use bevy::solari::{
    // pathtracer::{Pathtracer, PathtracingPlugin},
    prelude::{RaytracingMesh3d, SolariLighting, SolariPlugins},
};

#[cfg(feature = "solari")]
use wgpu::TextureUsages;

/// Make sure Entities with Camera3d + WorldCamera and ViewCamera exist,
/// reusing but reconfiguring any existing entities.
pub(crate) fn ensure_3d_camera(
    mut commands: Commands,
    world_camera_q: Query<Entity, (With<Camera3d>, With<WorldCamera>)>,
    view_camera_q: Query<Entity, (With<Camera3d>, With<ViewerCamera>)>,
    // camera_fx_q: Query<&CameraEffects, With<LevelRoot>>,
    render_device: Res<RenderDevice>,
    render_adapter: Res<RenderAdapter>,
) {
    let use_clustered =
        bevy::pbr::decal::clustered::clustered_decals_are_usable(&render_device, &render_adapter);

    let ent = if let Ok(ent) = world_camera_q.single() {
        // Got one.
        ent
    } else {
        info!("Creating 3D camera");

        commands.spawn_empty().id()
    };

    configure_world_camera(commands.get_entity(ent).unwrap(), use_clustered);
    // if let Ok(fx) = camera_fx_q.single() {
    //     configure_camera_effects(commands.get_entity(ent).unwrap(), fx, true);
    // } else {
    //     log::warn!("missing CameraEffects on scene");
    // }

    ////

    let ent = if let Ok(ent) = view_camera_q.single() {
        // Got one.
        ent
    } else {
        info!("Creating viewer camera");

        commands.spawn_empty().id()
    };

    configure_viewer_camera(commands.get_entity(ent).unwrap(), use_clustered);
    // if let Ok(fx) = camera_fx_q.single() {
    //     configure_camera_effects(commands.get_entity(ent).unwrap(), fx, false);
    // } else {
    //     log::warn!("missing CameraEffects on scene");
    // }

    ////

    // Force init.
    commands.insert_resource(VideoCameraSettingsChanged);
    commands.insert_resource(VideoEffectSettingsChanged);
}

fn configure_world_camera(mut ent_commands: EntityCommands, use_clustered: bool) {
    ent_commands.insert((
        (
            DespawnOnExit(GameplayState::Playing),

            (
                Name::new("WorldCamera"),
                WorldCamera,
                Camera3d::default(),
                RenderLayers::layer(RENDER_LAYER_DEFAULT),

                Exposure { ev100: 10.0 },
                Camera {
                    order: 1,
                    clear_color: Color::BLACK.into(),
                    ..default()
                },

                Hdr,

                Projection::Perspective(PerspectiveProjection {
                    fov: 75f32.to_radians(),
                    far: 1000.0,
                    ..default()
                }),

                #[cfg(not(target_arch = "wasm32"))]
                OrderIndependentTransparencySettings::default(),
                Msaa::Off,

                DepthPrepass,
                OcclusionCulling,
            ),
            (
                PlayerCamera(CameraMode::FirstPerson),
                OurCamera::default(),
                Transform::from_xyz(0., 1., 0.),
            ),

            // Audio is from the perspective of the camera.
            SpatialListener3D::default(),
        ),
    ));

     #[cfg(feature = "solari")]
    ent_commands.insert((
        // Msaa::Off and CameraMainTextureUsages with STORAGE_BINDING are required for Solari
        CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING),
        Msaa::Off,

        SolariLighting::default(),
    ));

    if !use_clustered {
        ent_commands.insert(DepthPrepass);
    }
}

fn configure_viewer_camera(mut ent_commands: EntityCommands, use_clustered: bool) {
    ent_commands.insert((
        (
            DespawnOnExit(GameplayState::Playing),

            Name::new("ViewCamera"),
            ViewerCamera,
            Camera3d::default(),
            RenderLayers::layer(RENDER_LAYER_VIEW),

            Exposure { ev100: 1.0 },
            Camera {
                order: 2,
                clear_color: ClearColorConfig::None,
                ..default()
            },

            Projection::Perspective(PerspectiveProjection {
                fov: 90f32.to_radians(),
                ..default()
            }),

            Hdr,
            Msaa::Off,  // must match WorldCamera
        ),
    ));

     #[cfg(feature = "solari")]
    ent_commands.insert((
        // Msaa::Off and CameraMainTextureUsages with STORAGE_BINDING are required for Solari
        CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING),
        Msaa::Off,

        SolariLighting::default(),
    ));

    if !use_clustered {
        // ent_commands.insert(DepthPrepass);
    }
}

// pub(crate) fn configure_camera_effects(mut ent_commands: EntityCommands, fx: &CameraEffects, is_world: bool) {
//     match fx {
//         CameraEffects::Normal => {
//             ent_commands.insert(Tonemapping::BlenderFilmic);
//             ent_commands.insert(Bloom::default());
//             ent_commands.insert(ColorGrading::default());
//         }
//         CameraEffects::Mode1 => {
//             ent_commands.insert(Tonemapping::TonyMcMapface);
//             if is_world || cfg!(not(target_arch = "wasm32")) {
//                 ent_commands.insert(Bloom {
//                     intensity: -1.0,
//                     low_frequency_boost: 1.0,
//                     low_frequency_boost_curvature: 0.0,
//                     high_pass_frequency: 1.0,
//                     ..default()
//                 });
//             } else {
//                 // Can't stack two Blooms well in webgl
//                 ent_commands.insert(Bloom {
//                     intensity: 0.,
//                     low_frequency_boost: 1.0,
//                     low_frequency_boost_curvature: 0.0,
//                     high_pass_frequency: 1.0,
//                     ..default()
//                 });
//             }
//             ent_commands.insert(ColorGrading {
//                 global: ColorGradingGlobal {
//                     exposure: 1.25,
//                     post_saturation: 1.5,
//                     ..default()
//                 },
//                 shadows: ColorGradingSection {
//                     lift: -0.005,
//                     ..default()
//                 },
//                 midtones: ColorGradingSection::default(),
//                 highlights: ColorGradingSection {
//                     lift: -0.005,
//                     ..default()
//                 }
//             });
//         }
//         CameraEffects::Mode2 => {
//             ent_commands.insert(Tonemapping::TonyMcMapface);
//             if is_world || cfg!(not(target_arch = "wasm32")) {
//                 ent_commands.insert(
//                     Bloom {
//                         intensity: -1.0,
//                         low_frequency_boost: 1.0,
//                         low_frequency_boost_curvature: 0.25,
//                         // high_pass_frequency: 1.0,
//                         scale: Vec2::new(0.5, 1.0),
//                         max_mip_dimension: 1024,
//                         ..default()
//                     }
//                 );
//             } else {
//                 // Can't stack two Blooms well in webgl
//                 ent_commands.insert(Bloom {
//                     intensity: 0.,
//                     low_frequency_boost: 1.0,
//                     low_frequency_boost_curvature: 0.25,
//                     // high_pass_frequency: 1.0,
//                     scale: Vec2::new(0.5, 1.0),
//                     ..default()
//                 });
//             }
//             ent_commands.insert(ColorGrading {
//                 global: ColorGradingGlobal {
//                     // exposure: 1.25,
//                     exposure: 1.0,
//                     post_saturation: 1.5,
//                     ..default()
//                 },
//                 shadows: ColorGradingSection {
//                     lift: -0.005,
//                     ..default()
//                 },
//                 midtones: ColorGradingSection::default(),
//                 highlights: ColorGradingSection {
//                     lift: -0.005,
//                     ..default()
//                 }
//             });
//         }
//     }
// }
