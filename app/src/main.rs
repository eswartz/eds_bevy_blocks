#![feature(iter_array_chunks)]

mod menus;
mod assets;
mod audio;
mod player_spawning;
mod actions;
mod camera;
mod game;

use crate::assets::*;
use crate::audio::AudioPlugin;
use crate::camera::ensure_3d_camera;
use crate::game::GamePlugin;
use crate::menus::MenuPlugin;

use std::time::Duration;

use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::{
    asset::AssetMetaCheck,
    image::{ImageAddressMode, ImageSamplerDescriptor},
    winit::WinitSettings,
};

use avian3d::dynamics::solver::SolverConfig;
use bevy::light::NotShadowCaster;
use bevy_asset_loader::prelude::*;
use bevy_skein::SkeinPlugin;

#[cfg(feature = "input_lim")]
use leafwing_input_manager::prelude::*;

use eds_bevy_common::*;

#[cfg(target_arch = "wasm32")]
use console_log;

fn main() -> AppExit {
    let res = find_runtime_base_directory_by_folder("assets");
    let base_dir = match res {
        Err(e) => {
            log::error!("startup failure: {e}");
            return AppExit::from_code(3);
        }
        Ok(base_dir) => base_dir,
    };

    #[cfg(target_arch = "wasm32")]
    let _ = console_log::init_with_level(log::Level::Info);

    let mut app = App::new();
    app
        // .insert_resource(WinitSettings::default())
        .insert_resource(WinitSettings {
            focused_mode: bevy::winit::UpdateMode::reactive_low_power(Duration::from_secs_f32(
                1.0 / 120.0,
            )),
            unfocused_mode: bevy::winit::UpdateMode::reactive_low_power(Duration::from_secs_f32(
                1.0 / 15.0,
            )),
        })

        // Register our sources early.
        .add_plugins(CommonAssetsPlugin)

        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    watch_for_changes_override: Some(true),
                    file_path: dbg!(base_dir.join("assets").display().to_string()),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        address_mode_w: ImageAddressMode::Repeat,
                        ..ImageSamplerDescriptor::linear()
                    },
                }),
            SkeinPlugin::default(),
        ))

        .add_plugins(PhysicsPlugins::default()
            .with_collision_hooks::<GeometryCollisionHooks>()
            .build()
            // .disable::<IslandPlugin>()
            // .disable::<IslandSleepingPlugin>()
        )
        .insert_resource(SubstepCount(8))

        .insert_resource(SolverConfig {
            contact_damping_ratio: 10.0,
            contact_frequency_factor: 3.0,
            max_overlap_solve_speed: 10.0,
            restitution_threshold: 1.0,
            ..default()
        })
        .add_plugins(avian3d::debug_render::PhysicsDebugPlugin::default())

        .insert_gizmo_config(
             PhysicsGizmos {
                 aabb_color: Some(Color::WHITE),
                 ..default()
             },
            GizmoConfig {
                // enabled: true,
                enabled: false,
                depth_bias: -0.1,
                ..default()
            },
        )

        .insert_resource(TimeToSleep(0.05))

        .add_plugins(AppPlugin)

        .add_plugins(ActionPlugin)

        .add_plugins(MenuPlugin)
        .add_plugins(LifecyclePlugin)
        .add_plugins(GuiPlugin)
        .add_plugins(WorldUiPlugin)
        .add_plugins(WorldStatePlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(CrosshairPlugin)
        .add_plugins(EffectsPlugin)
        .add_plugins(SkyboxPlugin)
        .add_plugins(LevelsPlugin)
        .add_plugins(DeathboxPlugin::default())

        .add_plugins(PlayerCameraPlugin)
        .add_plugins(PlayerInputPlugin)
        .add_plugins(PlayerClientPlugin)
        .add_plugins(PlayerMovementPlugin)
        .add_plugins(PlayerControllerPlugin)

        .insert_resource(OurUser(default()))
        .insert_resource(PlayerMode::Fps)
        .insert_resource(PlayerInputSettings::for_fps())
        // .insert_resource(PlayerMode::Space)
        // .insert_resource(PlayerInputSettings {
        //     base_xz_speed: 32,
        //     max_xz_speed: 255,
        //     max_down_speed: 255,
        //     max_up_speed: 255,
        //     accelerate_scale: 5.0,
        //     ..PlayerInputSettings::for_space()
        // })

        .add_loading_state(
            LoadingState::new(ProgramState::Initializing)
                .continue_to_state(ProgramState::New)
                .on_failure_continue_to_state(ProgramState::Error)
                .load_collection::<GuiAssets>()
                .load_collection::<SkyboxAssets>()
                .load_collection::<MapAssets>()
                .load_collection::<ModelAssets>()
                .load_collection::<ScriptAssets>()
        )
        .add_systems(
            OnEnter(GameplayState::Playing),
            (
                ensure_3d_camera,
                fixup_light_shadows,
            )
        )

        .insert_resource(ProductName("Blocks".to_string()))

        .add_systems(OnEnter(OverlayState::GameOverScreen),
            on_game_over_screen)
        .add_systems(OnExit(OverlayState::GameOverScreen),
            on_game_over_screen_finished)

        .add_plugins(GamePlugin)
    ;

    #[cfg(feature = "input_lim")]
    app.insert_resource(create_input_map());
    #[cfg(feature = "input_bei")]
    app.add_systems(Startup, create_input_map);

    if dev_tools_enabled() {
        app
            .add_plugins(DebugPlugin)
            .add_systems(
                First,
                (
                    bevy::dev_tools::states::log_transitions::<ProgramState>,
                    bevy::dev_tools::states::log_transitions::<GameplayState>,
                    bevy::dev_tools::states::log_transitions::<OverlayState>,
                    bevy::dev_tools::states::log_transitions::<LevelState>,
                ),
            )
            .add_plugins(crate::stats::StatsOverlayPlugin)
            .insert_resource(GuiState {
                show_fps: true,
                ..default()
            })
        ;
    }

    app.run()
}

#[cfg(feature = "input_lim")]
fn create_input_map() -> InputMap::<UserAction> {
    let mut map = InputMap::default();
    map.merge(&default_gui_input_map());
    map.merge(&default_fps_input_map());
    map.merge(&actions::extra_input_map());
    map
}

#[cfg(feature = "input_bei")]
fn create_input_map(mut commands: Commands) {
    use bevy_enhanced_input::prelude::ActionOf;

    use crate::actions::assign_extra_actions;

    let menu_entity = commands.spawn((
        MenuContext,
        Name::new("MenuContext"),
    )).id();
    let include = (
        ActionOf::<MenuContext>::new(menu_entity),
        MenuAction,
    );
    assign_stock_common_actions(commands.reborrow(), include.clone());
    assign_stock_menu_actions(commands.reborrow(), include.clone());

    ///////

    let player_entity = commands.spawn((
        PlayerContext,
        Name::new("PlayerContext"),
    )).id();

    let include = (
        ActionOf::<PlayerContext>::new(player_entity),
        PlayerAction,
    );
    assign_stock_common_actions(commands.reborrow(), include.clone());
    assign_stock_player_actions(commands.reborrow(), include.clone());
    assign_extra_actions(commands.reborrow(), include.clone());
}


#[derive(Component)]
pub(crate) struct GameOverScreen;

pub(crate) fn on_game_over_screen(
    mut commands: Commands,
    fonts: Option<Res<CommonGuiAssets>>,
) {
    let ent_commands = commands.spawn((
        Name::new("GameOver"),
        GameOverScreen,
    ));
    setup_game_over_screen(ent_commands, fonts.as_deref());
}

pub(crate) fn on_game_over_screen_finished(
    mut commands: Commands,
    gui_q: Query<Entity, With<GameOverScreen>>,
) {
    for ent in gui_q.iter() {
        commands.entity(ent).try_despawn();
    }
}

pub(crate) fn setup_game_over_screen(
    mut ent_commands: EntityCommands,
    fonts: Option<&CommonGuiAssets>,
) -> Entity {
    let font = fonts.map_or(default(), |fonts| fonts.std_ui.clone());
    ent_commands.insert((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            .. default()
        },
        BackgroundColor(tailwind::GREEN_800.with_alpha(0.5).into()),
        RenderLayers::from_layers(&[RENDER_LAYER_UI]),
    ))
    .with_children(|builder| {
        builder.spawn((
            Text::new(
                "You Won!",
            ),
            TextFont {
                font: font.clone(),
                font_size: 32.0,
                .. default()
            },
            TextColor(Color::WHITE.with_alpha(0.5)),
        ));
        builder.spawn((
            Text::new(
                "\u{a0}",
            ),
            TextFont {
                font: font.clone(),
                font_size: 32.0,
                .. default()
            },
            TextColor(Color::WHITE.with_alpha(0.5)),
        ));
        builder.spawn((
            Text::new(
                "Thanks for playing!",
            ),
            TextFont {
                font: font.clone(),
                font_size: 32.0,
                .. default()
            },
            TextColor(Color::WHITE.with_alpha(0.5)),
        ));
    })
    .id()
}


/// Make sure lights cast shadows.
pub(crate) fn fixup_light_shadows(
    mut light_q: ParamSet<(
        Query<&mut PointLight, Without<NotShadowCaster>>,
        Query<&mut SpotLight, Without<NotShadowCaster>>,
        Query<&mut DirectionalLight, Without<NotShadowCaster>>,
    )>,
) {
    for mut light in light_q.p0().iter_mut() {
        light.shadows_enabled = true;
    }
    for mut light in light_q.p1().iter_mut() {
        light.shadows_enabled = true;
    }
    for mut light in light_q.p2().iter_mut() {
        light.shadows_enabled = true;
    }
}
