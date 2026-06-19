
mod action_handlers;
mod sound;
mod script_debug;
mod bevy_funcs;
mod gravity_sleep;
mod level_0;
mod level_1;
mod level_2;
mod level_3;
mod level_4;
mod level_5;
mod level_6;

use avian3d::math::*;
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::tailwind;
use bevy::ecs::query::QueryData;
use bevy::gltf::GltfMesh;
use bevy::mesh::*;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::platform::collections::HashMap;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use fedry_bevy_plugin::prelude::{FedryScriptingPlugin, ScriptRoot, pause_scripting, register_script_key, unpause_scripting};
pub use action_handlers::*;
use fedry_runtime::prelude::RuntimeError;
use strum::{EnumIter, VariantArray};

use std::time::Duration;

use crate::assets::ModelAssets;
use crate::game::bevy_funcs::register_funcs;
use crate::game::gravity_sleep::GravitySleepPlugin;
use crate::game::script_debug::ScriptDebugPlugin;
use crate::game::sound::SoundPlugin;
use eds_bevy_common::*;

use bevy::asset::uuid::Uuid;
use bevy::ecs::world::CommandQueue;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::{
    scene::SceneInstanceReady,
};

use eds_bevy_common::midi_synth::prelude::*;

// #[cfg(all(feature = "solari", feature = "dlss"))]
// use bevy::anti_alias::dlss::{
//     Dlss, DlssProjectId, DlssRayReconstructionFeature, DlssRayReconstructionSupported,
// };
#[cfg(feature = "solari")]
use bevy::solari::{
    prelude::{RaytracingMesh3d, SolariPlugins},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {

        #[cfg(feature = "solari")]
        {
            app.add_plugins(SolariPlugins);
        }

        app
            .add_plugins(ActionHandlersPlugin)
            .add_plugins(SoundPlugin)
            .add_plugins(FedryScriptingPlugin)
            .add_plugins(ScriptDebugPlugin)
            .add_plugins(GravitySleepPlugin)

            .add_plugins(DetailNormalPlugin)
            .add_plugins(SplitIntoCubesPlugin)

            .insert_resource(Gravity((9.8 * Vec3::NEG_Y).into()))

            .add_systems(
                PreUpdate,
                    // wake_up_spawned_if_floating.run_if(resource_changed::<Gravity>)
                    wake_up_spawned.run_if(resource_changed::<Gravity>)
                    .run_if(not(is_in_menu))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )

            .insert_resource(GrabbingBehavior {
                ignore_mass: true,
                .. default()
            })

            .init_resource::<LevelDifficulty>()

            .add_plugins(level_0::LevelPlugin)
            .add_plugins(level_1::LevelPlugin)
            .add_plugins(level_2::LevelPlugin)
            .add_plugins(level_3::LevelPlugin)
            .add_plugins(level_4::LevelPlugin)
            .add_plugins(level_5::LevelPlugin)
            .add_plugins(level_6::LevelPlugin)

            .insert_resource(BaseEntity(Entity::PLACEHOLDER, Transform::IDENTITY))

            .add_observer(on_scene_ready)

            .add_systems(
                OnExit(ProgramState::New),
                ensure_levels
            )
            .add_systems(
                OnEnter(GameplayState::Setup),
                (
                    level_spawn_started,
                    spawn_level,
                ).chain()
            )
            .add_systems(
                OnExit(GameplayState::Setup),
                (
                    level_spawn_finished,
                ).chain()
            )

            .add_systems(
                PreUpdate,
                add_raytracing_to_meshes,
            )

            .add_systems(
                FixedUpdate,
                (
                    init_player_settings,
                    spawn_player_on_start,
                )
                .chain()
                .run_if(added_player_start) // <<< only once per session, in practice
                .run_if(in_state(GameplayState::Playing))
            )

            .add_systems(
                Startup,
                register_funcs,
            )
            .add_systems(OnEnter(LevelState::LevelLoaded),
                (
                    |mut commands: Commands| {
                        commands.set_state(LevelState::Configuring);
                    },
                    setup_skybox,
                    unpause_scripting,
                ).chain()
            )
            .add_systems(OnExit(LevelState::Playing),
                (
                    pause_scripting,
                )
            )
            .add_systems(
                Update,
                (
                    report_raycast,
                )
                .run_if(not(is_paused))
                .run_if(not(is_in_menu))
                .run_if(is_level_active)
                .run_if(not(debug_gui_wants_direct_input))
                .run_if(in_state(LevelState::Playing))
                .run_if(in_state(ProgramState::InGame))
            )

            .add_systems(OnEnter(LevelState::Playing),
                show_power_bar
            )
            .add_systems(OnExit(LevelState::Playing),
                remove_power_bar
            )
            .add_systems(
                Update,
                update_power_bar
                    .run_if(not(is_paused))
                    .run_if(in_state(ProgramState::InGame))
                    .run_if(in_state(GameplayState::Playing))
            )

            .add_systems(
                OnEnter(LevelState::Won),
                won_level,
            )
            .add_systems(
                OnEnter(LevelState::Lost),
                lost_level
            )

            .add_systems(
                OnEnter(LevelState::Advance),
                advance_level
            )

            .add_systems(
                FixedUpdate,
                (
                    update_current_score,
                )
                    .run_if(not(is_in_menu))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )

            .add_systems(
                FixedUpdate,
                (
                    check_next_level,
                    check_won_level.run_if(in_state(LevelState::Won)),
                    check_lost_level.run_if(in_state(LevelState::Lost)),
                )
                    .run_if(not(is_in_menu))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )

            .add_systems(First,
                spawn_midi_synths.run_if(resource_exists::<CommonSoundFontAssets>)
                    .run_if(not(is_in_menu))
                    .run_if(in_state(ProgramState::InGame)),
            )

            .insert_resource(PaletteMaterialHandles(default()))
            .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, PaletteMaterialExtension>>::default())
            .add_systems(PreUpdate, handle_palette)

            .insert_resource(DepthMapStorage { orig_to_edited: default() })
            .add_systems(PreUpdate, (
                handle_depth_map,
                tick_depth_map,
            ).chain())

            .add_systems(PreUpdate, apply_uv_box_map)

            .add_systems(OnEnter(LevelState::Advance), cleanup_palette_materials)
        ;

        register_script_key::<GameScript>(app);
    }
}

#[derive(Resource, Default)]
pub struct PaletteMaterialHandles
(
    HashMap<
        Handle<StandardMaterial>,
        Handle<ExtendedMaterial<StandardMaterial, PaletteMaterialExtension>>,
    >,
);

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct PaletteMaterialExtension {
}

const PM_SHADER_ASSET_PATH: &str = "shaders/palette_material.wgsl";

impl MaterialExtension for PaletteMaterialExtension {
    // fn vertex_shader() -> ShaderRef {
    //     PM_SHADER_ASSET_PATH.into()
    // }

    fn fragment_shader() -> ShaderRef {
        PM_SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        PM_SHADER_ASSET_PATH.into()
    }
}

#[derive(Component, Debug, Default, Clone, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component, Default, Clone)]
#[type_path = "fedry"]
pub struct SpawnPaletteMaterial;

pub(crate) fn handle_palette(
    mut commands: Commands,
    mut pal_mats: ResMut<Assets<ExtendedMaterial<StandardMaterial, PaletteMaterialExtension>>>,
    mut pal_mat_cache: ResMut<PaletteMaterialHandles>,

    std_mats: Res<Assets<StandardMaterial>>,

    mat_q: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<SpawnPaletteMaterial>>,
) -> Result {
    for (entity, std_mat_handle) in mat_q.iter() {
        let mut ent_commands = commands.entity(entity);

        let std_mat = std_mats.get(std_mat_handle.id())
            .ok_or_else(|| RuntimeError::LiteralError(
                format!("no StandardMaterial on {entity}")
            ))?;
        let pal_mat = pal_mat_cache.0
            .entry(std_mat_handle.0.clone())
            .or_insert_with(|| {
                pal_mats.add(ExtendedMaterial {
                    base: std_mat.clone(),
                    // base: StandardMaterial {
                    //     unlit: true,
                    //     base_color_texture: Some(assets.load("textures/palette.png")),
                    //     ..default()
                    // },
                    extension: default(),
                })
            });

        ent_commands.remove::<(
            SpawnPaletteMaterial,
            MeshMaterial3d<StandardMaterial>,
        )>();

        ent_commands.insert(MeshMaterial3d(pal_mat.clone()));
    }

    Ok(())
}

/// Current difficulty.
#[derive(Resource, Default, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub struct LevelDifficulty(pub Difficulty);

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct BoomMass(f32);

impl std::ops::Deref for BoomMass {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Marker for scripts driven by the game itself.
#[derive(Debug, Default, Clone, PartialEq, Hash, Reflect, QueryData)]
#[type_path = "fedry"]
pub(crate) struct GameScript;

/// Difficulty rating.
#[derive(
    Resource,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    Reflect,
    EnumIter,
    strum_macros::Display,
    VariantArray,
)]
#[reflect(Resource)]
#[type_path = "game"]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

/// The current score.
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource, Default)]
#[type_path = "game"]
pub struct CurrentScore {
    pub score: i32,
}

const END_LEVEL_DELAY_SECS: u64 = 3;

/// Countdown to next or same level.
#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
#[type_path = "game"]
pub struct AutoEndLevelTimer {
    pub(crate) timer: Timer,
}

impl AutoEndLevelTimer {
    pub fn new(delay: Duration) -> Self {
        Self {
            timer: Timer::new(delay, TimerMode::Once),
        }
    }
}

/// A cube.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Clone, Default)]
#[type_path = "game"]
pub(crate) struct Cube;

/// A floor.
#[derive(Component, Debug, Reflect, Default, Clone)]
#[reflect(Component, Clone, Default)]
#[type_path = "game"]
pub(crate) struct Floor;

// World state

/// Our "base" object and its initial transform.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct BaseEntity(pub Entity, pub Transform);


// Player state

/// Marker for an object (e.g. net) in the hand.
#[derive(Component)]
#[allow(unused)]
pub(crate) struct InHand;

fn on_scene_ready(
    ready: On<SceneInstanceReady>,
    children_q: Query<&Children>,
    meshes_q: Query<&Mesh3d, Without<CollisionLayers>>,
    mut commands: Commands,
) {
    for entity in children_q.iter_descendants(ready.entity) {
        if meshes_q.contains(entity) {
            commands.entity(entity).insert((
                CollisionLayers::new(
                    GameLayer::World,
                    [
                        GameLayer::Default,
                        GameLayer::World,
                        GameLayer::Player,
                        GameLayer::Projectiles,
                    ],
                ),
            ));
        }
    }
}

#[allow(unused)]
fn extract_mesh_cube(mesh: &Mesh, center: Vec3, half_size: Vec3) -> Option<(Mesh, Vec<[u32; 3]>, Vec<Vector>)> {
    let inds = mesh.indices().unwrap();

    let full_pos = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap();
    let full_normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap().as_float3().unwrap();
    let full_uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() {
        VertexAttributeValues::Float32x2(values) => values,
        _ => panic!(),
    };

    let mut pos = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];
    for [ind0, ind1, ind2] in inds.iter().array_chunks::<3>() {
        let pos0 = full_pos[ind0];
        let pos1 = full_pos[ind1];
        let pos2 = full_pos[ind2];
        if contains_pt(&pos0, center, half_size)
        || contains_pt(&pos1, center, half_size)
        || contains_pt(&pos2, center, half_size) {
            let l = pos.len() as u32;
            indices.push([l, l + 1, l + 2]);

            pos.push(pos0);
            pos.push(pos1);
            pos.push(pos2);

            normals.push(full_normals[ind0]);
            normals.push(full_normals[ind1]);
            normals.push(full_normals[ind2]);

            uvs.push(full_uvs[ind0]);
            uvs.push(full_uvs[ind1]);
            uvs.push(full_uvs[ind2]);
        }
    }

    if pos.is_empty() {
        return None
    }

    let mut mesh = Mesh::new(wgpu::PrimitiveTopology::TriangleList, RenderAssetUsages::all())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(pos.clone()))
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals))
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

    if let Err(err) = mesh.generate_tangents() {
        warn!("failed to generate tangents: {err}");
    }

    // Some(mesh)

    let positions = pos
        .into_iter()
        .map(|v| Vec3::from_array(v)
            .into())
        .collect::<Vec<_>>();
    Some((mesh, indices, positions))
}

fn contains_pt(pt: &[f32; 3], center: Vec3, half_size: Vec3) -> bool {
    pt[0] >= center.x - half_size.x && pt[0] <= center.x + half_size.x
    && pt[1] >= center.y - half_size.y && pt[1] <= center.y + half_size.y
    && pt[2] >= center.z - half_size.z && pt[2] <= center.z + half_size.z
}

pub(crate) fn ensure_levels(mut level_list: ResMut<LevelList>) {
    level_list.0.sort_by(|a, b| a.id.cmp(&b.id));
}

pub(crate) fn level_spawn_started(mut commands: Commands, mut pause: ResMut<PauseState>) {
    commands.set_state(LevelState::Initializing);
    commands.set_state(OverlayState::Loading);

    // Prevent moving/interacting while loading UI is up.
    pause.set_menu_paused(true);
}

pub(crate) fn level_spawn_finished(
    mut commands: Commands,
    mut pause: ResMut<PauseState>,
    sensable_q: Query<Entity, Or<(
        With<DeathboxCollider>,
    )>>,
    // coll_floor_q: Query<Entity, (With<Mesh3d>, With<Floor>),
    // floor_q: Query<Entity, (With<Floor>, With<RigidBody>),
    // floor_q: Query<Entity, (With<Floor>, Without<ColliderConstructor>)>,
) {
    for ent in sensable_q.iter() {
        commands.entity(ent).insert((
            Sensor,
            CollisionEventsEnabled,
            CollidingEntities::default(),
        ));
    }

    commands.set_state(OverlayState::Hidden);
    commands.set_state(LevelState::LevelLoaded);

    // Go for it, user (unless they did set_user_paused)
    pause.set_menu_paused(false);
}

#[cfg(not(feature = "solari"))]
fn add_raytracing_to_meshes() {}

#[cfg(feature = "solari")]
fn add_raytracing_to_meshes(
    world: If<Res<WorldMarkerEntity>>,
    children: Query<&Children>,
    mesh_query: Query<(
        &Mesh3d,
    ), Changed<Mesh3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if mesh_query.is_empty() {
        return
    }

    for descendant in children.iter_descendants(world.0.0) {
        if let Ok((Mesh3d(mesh_handle),)) =
            mesh_query.get(descendant)
        {
            // Add raytracing mesh component
            commands
                .entity(descendant)
                .insert(RaytracingMesh3d(mesh_handle.clone()));

            // Ensure meshes are Solari compatible
            let mesh = meshes.get_mut(mesh_handle).unwrap();
            if !mesh.contains_attribute(Mesh::ATTRIBUTE_UV_0) {
                let vertex_count = mesh.count_vertices();
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertex_count]);
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_TANGENT,
                    vec![[0.0, 0.0, 0.0, 0.0]; vertex_count],
                );
            }
            if !mesh.contains_attribute(Mesh::ATTRIBUTE_TANGENT) {
                mesh.generate_tangents().unwrap();
            }
            if mesh.contains_attribute(Mesh::ATTRIBUTE_UV_1) {
                mesh.remove_attribute(Mesh::ATTRIBUTE_UV_1);
            }
            if let Some(indices) = mesh.indices_mut()
                && let Indices::U16(_) = indices
            {
                *indices = Indices::U32(indices.iter().map(|i| i as u32).collect());
            }
        }
    }
}


fn added_player_start(q: Query<&Transform, Added<PlayerStart>>) -> bool {
    let flag = q.iter().next().is_some();
    flag
}

pub(crate) fn spawn_player_on_start(world: &mut World) {
    let mut start_q = world.query_filtered::<&Transform, With<PlayerStart>>();
    let Some(xfrm) = start_q.iter(world).next() else {
        log::error!("no PlayerStart");
        return;
    };

    drop(start_q);
    let xfrm = xfrm.clone();

    // Make the player collision model and Player
    let player_ent = spawn_fps_player(world, Uuid::default(),
        Vec3::new(0.5, 1.5, 0.3),
        xfrm.clone());

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);

    // Put and orient the new Player where the PlayerStart is.
    commands.entity(player_ent).insert((
        PlayerLook { rotation: xfrm.rotation, .. default() },
        xfrm
    ));

    // // Silliness
    // commands.spawn((
    //     ChildOf(player_ent),
    //     Transform::from_translation(Vec3::new(0., 10.0, 0.0)),
    //     PointLight {
    //         color: bevy::prelude::Color::Srgba(tailwind::AMBER_50 * 5.0f32),
    //         range: 15.0,
    //         intensity: 1.0e5,
    //         .. default()
    //     },
    //     Visibility::Visible,
    // ));

    queue.apply(world);
}

pub(crate) fn setup_level(
    mut commands: Commands,
    level_list: &LevelList,
    level_index: &LevelIndex,
) {
    let index = level_index.0;
    if index >= level_list.0.len() {
        log::error!("no items in LevelList");
        commands.remove_resource::<CurrentLevel>();
        commands.set_state(ProgramState::Error);
        return;
    }

    let level = &level_list.0[level_index.0];
    commands.insert_resource(CurrentLevel(level.clone()));
}

pub(crate) fn spawn_level(
    mut commands: Commands,
    level_list: Res<LevelList>,
    level_index: Res<LevelIndex>,
    world: Res<WorldMarkerEntity>,
    mut script_root: ResMut<ScriptRoot>,
) {
    setup_level(commands.reborrow(), &level_list, &level_index);

    if level_index.0 >= level_list.0.len() {
        return;
    }

    let level = &level_list.0[level_index.0];
    log::info!("Entering level {}", level.label);

    // Put any script content here.
    script_root.0 = world.0;

    commands
        .spawn((
            DespawnOnExit(GameplayState::Playing),
            SceneRoot(level.scene.clone()),
            ChildOf(world.0),
        ))
        .observe(|_event: On<SceneInstanceReady>, mut commands: Commands,| {
            commands.set_state(GameplayState::Playing);
        })
    ;

    commands.insert_resource(CurrentScore::default());
}

fn init_player_settings(
    move_q: Query<&PlayerCameraMode, With<LevelRoot>>,
    mut commands: Commands,
    mut settings: ResMut<PlayerInputSettings>,
) {
    if let Ok(mode) = move_q.single() {
        match mode.0 {
            PlayerMode::Fps => *settings = PlayerInputSettings::for_fps(),
            PlayerMode::Space => *settings = PlayerInputSettings::for_space(),
        }
        commands.insert_resource(mode.0);
    } else {
        log::warn!("no PlayerCameraMode in LevelRoot");
    }
}

pub(crate) fn advance_level(
    mut commands: Commands,
) {
    commands.set_state(OverlayState::Loading);
    commands.set_state(GameplayState::Setup);
}

fn update_current_score(
    mut commands: Commands,
    level_state: Res<State<LevelState>>,
    score: Option<Res<CurrentScore>>,
    gui_area: GuiAreaMarkerLocator,
    // mut score_q: Single<(&mut Text, &mut TextColor), With<ScoreArea>>,
    mut score_q: Query<(&mut Text, &mut TextColor)>,
) {
    gui_area.with_first(GuiAreaMarker::ScoreArea, |ent| {
        let Ok((ref mut text, ref mut color)) = score_q.get_mut(ent) else { return };
        if score.is_some() {
            if *level_state == LevelState::Playing {
                // let won = score.score >= goal.goal as _;
                // let lost = score.score <= goal.lose;

                let won = false;
                let lost = false;
                text.0 = String::new();
                color.0 = Color::Srgba(if won {
                    tailwind::LIME_300
                } else if lost {
                    tailwind::RED_700
                } else {
                    tailwind::GRAY_100
                });

                if won {
                    commands.set_state(LevelState::Won);
                } else if lost {
                    commands.set_state(LevelState::Lost);
                }
            }
        } else {
            text.0.clear();
        }
    });
}

/// Apply the [NextLevelIndex] value, if set.
fn check_next_level(
    mut level_index: ResMut<LevelIndex>,
    next_level_index_opt: Option<ResMut<NextLevelIndex>>,
    mut commands: Commands,
) {
    next_level_index_opt.map(|next_level| {
        commands.remove_resource::<NextLevelIndex>();
        *level_index = LevelIndex(next_level.0);
        commands.set_state(ProgramState::InGame);
    });
}

fn won_level(
    mut commands: Commands,
    gui_area: GuiAreaMarkerLocator,
    mut score_q: Query<(&mut Text, &mut TextColor)>,
) {
    gui_area.with_first(GuiAreaMarker::ScoreArea, |ent| {
        let Ok((ref mut text, ref mut color)) = score_q.get_mut(ent) else { return };
        text.0 = "Passed!".to_string();
        color.0 = Color::Srgba(tailwind::LIME_300);
    });
    commands.insert_resource(AutoEndLevelTimer::new(Duration::from_secs(END_LEVEL_DELAY_SECS)));
}

fn lost_level(
    mut commands: Commands,
    gui_area: GuiAreaMarkerLocator,
    mut score_q: Query<(&mut Text, &mut TextColor)>,
) {
    gui_area.with_first(GuiAreaMarker::GameStatusArea, |ent| {
        let Ok((ref mut text, ref mut color)) = score_q.get_mut(ent) else { return };
        text.0 = "Failed...\nTry again!".to_string();
        color.0 = Color::Srgba(tailwind::RED_700);
    });
    commands.insert_resource(AutoEndLevelTimer::new(Duration::from_secs(END_LEVEL_DELAY_SECS)));
}

fn check_won_level(
    mut commands: Commands,
    mut end_timer: ResMut<AutoEndLevelTimer>,
    time: Res<Time>,
    level_index: ResMut<LevelIndex>,
    level_list: Res<LevelList>,
) {
    if !end_timer.timer.tick(time.delta()).is_finished() {
        return;
    }

    let next_index = level_index.0 + 1;
    if next_index >= level_list.0.len() {
        commands.set_state(ProgramState::Completed);
        commands.set_state(LevelState::Initializing);
        commands.set_state(GameplayState::Done);
        commands.set_state(OverlayState::GameOverScreen);
        // Next time we restart, be at level 0.
        commands.insert_resource(LevelIndex(0));
    } else {
        commands.insert_resource(NextLevelIndex(next_index));
        commands.set_state(LevelState::Advance);
    }
}

fn check_lost_level(
    mut commands: Commands,
    mut end_timer: ResMut<AutoEndLevelTimer>,
    time: Res<Time>,
) {
    if !end_timer.timer.tick(time.delta()).is_finished() {
        return;
    }

    // Restarts level.
    commands.set_state(LevelState::Advance);
}

/// The power bar image inside [HandStatusArea].
#[derive(Component)]
pub struct PowerBarImage;

/// The power bar text inside [HandStatusArea].
#[derive(Component)]
pub struct PowerBarText;

fn show_power_bar(
    mut commands: Commands,
    gui_area: GuiAreaMarkerLocator,
    assets: Res<CommonGuiAssets>,
) {
    gui_area.with_first(GuiAreaMarker::HandStatusArea, |ent| {
        commands.entity(ent)
            .insert(UiNodeAlpha(0.0))
            .with_children(|builder| {
            builder.spawn((
                Name::new("PowerBar"),
                PowerBarImage,
                Visibility::Inherited,
                UiNodeAlpha(1.0),
                ImageNode::new(assets.power_bar.clone())
                    .with_color(Color::WHITE),
                Node {
                    width: Val::Vw(10.),
                    max_width: Val::Vw(10.),
                    min_width: Val::Px(128.),
                    aspect_ratio: Some(4.0),
                    align_content: AlignContent::Stretch,
                    ..default()
                },
            ));
            builder.spawn((
                Name::new("InHandText"),
                PowerBarText,
                Visibility::Inherited,
                UiNodeAlpha(1.0),
                Node {
                    ..default()
                },
                TextFont {
                    font: assets.std_ui.clone(),
                    font_size: 24.0,
                    weight: FontWeight::BOLD,
                    .. default()
                },
                TextColor(Color::Srgba(tailwind::RED_700)),
                TextShadow {
                    offset: Vec2::splat(1.0),
                    color: Color::WHITE,
                },
                Text::new("POWER"),
            ));
        });
    });
}

fn remove_power_bar(
    mut commands: Commands,
    gui_area: GuiAreaMarkerLocator,
    child_q: Query<&Children>,
) {
    gui_area.with_first(GuiAreaMarker::HandStatusArea, |ent| {
        for kid in child_q.iter_descendants(ent) {
            commands.entity(kid).try_despawn();
        }
    });
}

fn update_power_bar(
    fire_power: Res<FirePower>,
    gui_area: GuiAreaMarkerLocator,
    mut alpha_q: Query<&mut UiNodeAlpha>,
) {
    if fire_power.is_changed() {
        gui_area.with_first(GuiAreaMarker::HandStatusArea, |ent| {
            let Ok(mut alpha) = alpha_q.get_mut(ent) else { return };
            alpha.0 = (**fire_power / 50.0).clamp(0.0, 1.0);
        });
    }
}

fn setup_skybox(
    mut commands: Commands,
    skybox_q: Query<Entity, (With<SkyboxModel>,)>,
    cam_q: Query<Entity, (With<Camera3d>, With<WorldCamera>)>,
    skyboxes: Res<CommonSkyboxAssets>,
) {
    let Ok(cam) = cam_q.single() else { return };

    // If there isn't one in the level, add a default.
    if skybox_q.is_empty() {
        //let with_reflection_probe = Some(SkyboxReflectionProbeModel::default());  // looks ... not so good when real lights are present
        let with_reflection_probe = None;

        commands.entity(cam).insert(SkyboxModel {
            image: Some(skyboxes.dresden_station_night.clone()),
            brightness: bevy::prelude::light_consts::lux::HALLWAY,
            mapping: CubemapMapping::PxNxPyFxFyNyFxFyPzNz,
            with_reflection_probe,
            .. default()
        });
    }
    commands.insert_resource(SkyboxSetup::WaitingSkybox);
}

/// When placed on an entity, ensure a [MidiSynth] is wired up
/// and can process events.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct OurMidiSynth;

pub(crate) fn spawn_midi_synths(
    mut commands: Commands,
    sf_assets: Res<CommonSoundFontAssets>,
    muted: Res<MidiSynthsPaused>,
    synth_q: Query<Entity, (With<OurMidiSynth>, Without<MidiSynth>)>,
    mut synth_map: ResMut<SynthProxyMap>,
) -> Result<()> {
    let params = MidiSynthParams::default().is_world_positioned(true);

    for ent in synth_q.iter() {
        let (sample_sender, sample_receiver) = crossbeam_channel::unbounded();
        let synth = MidiSynth::new(
            params.clone(),
            sf_assets.timgm6mb.clone(),
            muted.0.clone(),
            ent,
            sample_sender,
            sample_receiver,
        )?;
        commands.entity(ent).insert((
            synth,
            Sfx,
        ));

        synth_map.register_synth(ent);

        commands.write_message(SynthMessage::new(ent, SynthCommand::Reset));
    }

    Ok(())
}

/// Used to save off [GravityScale] for purposes of inducing sleeping.
#[derive(Component, Debug, Reflect, Clone, PartialEq)]
#[reflect(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct OrigGravityScale{
    pub orig_scale: Scalar,
    pub restore_next_time: bool,
}

fn report_raycast(
    gui_area: GuiAreaMarkerLocator,
    mut info_q: Query<(&mut Text, &mut TextColor, &mut Visibility)>,
    highlighting_mode: Res<HighlightingMode>,
    crosshair_target: Res<CrosshairTargets>,
    names_q: Query<Option<&Name>>,
    gui_state: Res<GuiState>,
    mut last_target_desc: Local<(CrosshairTargets, String)>,
) {
    if !dev_tools_enabled() {
        return
    }

    gui_area.with_first(GuiAreaMarker::InfoArea, |ent| {
        let Ok((ref mut text, ref mut color, ref mut visibility)) = info_q.get_mut(ent) else { return };
        if !highlighting_mode.is_disabled()
        && gui_state.enabled
        && !crosshair_target.targets.is_empty()
        && let Some(message) = if last_target_desc.0 == *crosshair_target {
                // Same as last tick.
                Some(last_target_desc.1.clone())
            } else {
                // Recompute.
                if let Some(message) = report_crosshair_targets(&crosshair_target, &names_q) {
                    *last_target_desc = (crosshair_target.clone(), message.clone());
                    Some(message)
                } else {
                    None
                }
            }
        {
            visibility.set_if_neq(Visibility::Inherited);
            text.0 = message;
            color.0 = Color::Srgba(tailwind::GRAY_100);
        } else {
            visibility.set_if_neq(Visibility::Hidden);
        }
    });
}

// /// Avian doesn't reliably wake up (or cause [RigidBody]s to move)
// /// when changing Gravity. Maybe that's intentional, maybe a bug?
// fn wake_up_spawned_if_floating(
//     mut commands: Commands,
//     collisions: Res<ContactGraph>,
//     sleep_q: Query<Entity, (With<Sleeping>, With<Spawned>, Without<Player>)>
// ) {
//     for ent in sleep_q.iter() {
//         if collisions.entities_colliding_with(ent).next().is_none() {
//             commands.entity(ent).remove::<Sleeping>();
//         }
//     }
// }

/// Avian doesn't reliably wake up (or cause [RigidBody]s to move)
/// when changing Gravity. Maybe that's intentional, maybe a bug?
fn wake_up_spawned(
    mut commands: Commands,
    sleep_q: Query<Entity, (With<Sleeping>, With<Spawned>, Without<Player>)>
) {
    for ent in sleep_q.iter() {
        commands.entity(ent).remove::<Sleeping>();
    }
}


pub fn load_cube_model(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    gltf_meshes: &Res<Assets<GltfMesh>>,
    model_assets: &Res<ModelAssets>,
) -> Result<(Handle<Mesh>, Handle<StandardMaterial>)> {
    // Spawn cube stacks
    let gmesh = gltf_meshes.get(&model_assets.cube).ok_or(format!("could not load cube"))?;

    let prim = &gmesh.primitives[0];
    let cube_mesh = prim.mesh.clone();
    let std_mat = if let Some(mat) = &prim.material {
        materials.get(mat).ok_or(format!("could not load cube material"))?.clone()
    } else {
        Into::<StandardMaterial>::into(Color::WHITE)
    };
    let mat = materials.add(std_mat);

    Ok((cube_mesh, mat))
}

fn cleanup_palette_materials(
    pal_mats: Option<ResMut<PaletteMaterialHandles>>,
) {
    if let Some(mut pal_mats) = pal_mats {
        pal_mats.0.clear();
    }
}
