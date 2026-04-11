
mod logic;
mod sound;
mod script_debug;
mod level_0;
// mod level_1;
// mod level_2;
// mod level_3;

use avian3d::math::Vector;
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::tailwind;
use bevy::ecs::query::QueryData;
use bevy::mesh::{VertexAttributeValues, triangle_normal};
use bevy_tweening::lens::TextColorLens;
use bevy_tweening::{AnimTarget, EaseMethod, Tween, TweenAnim};
use fedry_bevy_plugin::FedryScriptingPlugin;
use fedry_bevy_plugin::prelude::register_script_key;
pub use logic::*;
use strum::{EnumIter, VariantArray};

use std::time::Duration;

use crate::game::script_debug::ScriptDebugPlugin;
use crate::game::sound::SoundPlugin;
use eds_bevy_common::*;
use crate::player_spawning::spawn_player;

use bevy::asset::uuid::Uuid;
use bevy::ecs::world::CommandQueue;
use bevy_seedling::prelude::*;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::{
    scene::SceneInstanceReady,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(LogicPlugin)
            .add_plugins(SoundPlugin)
            .add_plugins(FedryScriptingPlugin)
            .add_plugins(ScriptDebugPlugin)

            .init_resource::<LevelDifficulty>()

            .add_plugins(level_0::LevelPlugin)
            // .add_plugins(level_1::LevelPlugin)
            // .add_plugins(level_2::LevelPlugin)
            // .add_plugins(level_3::LevelPlugin)

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
            // .add_systems(
            //     Update,
            //     update_bone_aabb_added
            //     // .run_if(in_state(GameplayState::Setup))
            // )
            .add_systems(
                OnExit(GameplayState::Setup),
                (
                    level_spawn_finished,
                ).chain()
            )
            .add_systems(
                Update,
                (
                    init_player_settings,
                    spawn_player_on_start,
                )
                .chain()
                .run_if(added_player_start) // <<< only once per session, in practice
                .run_if(in_state(GameplayState::Playing))
            )
            .add_systems(
                OnTransition{ exited: GameplayState::Playing, entered: GameplayState::Setup },
                (
                    hide_instructions,
                    despawn_level,
                )
            )

            .add_systems(OnEnter(LevelState::LevelLoaded),
                (
                    start_skybox_setup,
                    show_instructions,
                ).chain()
                    .run_if(in_state(ProgramState::InGame))
            )

            .add_systems(OnExit(LevelState::Playing),
                hide_instructions,
            )

            .add_systems(OnEnter(LevelState::Playing),
                show_power_bar
                    .run_if(in_state(ProgramState::InGame))
            )
            .add_systems(OnExit(LevelState::Playing),
                remove_power_bar
                    .run_if(in_state(ProgramState::InGame))
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

            // .add_observer(
            //     handle_grab_actions
            //         .run_if(not(is_paused))
            //         .run_if(not(is_in_menu))
            //         .run_if(in_state(LevelState::Playing))
            //         .run_if(in_state(ProgramState::InGame))
            //     ,
            // )
            .add_systems(
                Update,
                (
                    update_current_score,
                )
                    .run_if(not(is_in_menu))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )

            .add_systems(
                Update,
                (
                    check_won_level.run_if(in_state(LevelState::Won)),
                    check_lost_level.run_if(in_state(LevelState::Lost)),
                )
                    .run_if(not(is_in_menu))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )
        ;

        register_script_key::<ScriptMain>(app);
    }
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

#[derive(Debug, Clone, PartialEq, Hash, Reflect, QueryData)]
pub(crate) struct ScriptMain;

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
pub struct AutoEndLevelTimer(pub(crate) Timer);

/// A cube.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Clone, Default)]
#[type_path = "game"]
pub(crate) struct Cube;

/// A floor.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Clone, Default)]
#[type_path = "game"]
pub(crate) struct Floor;


// World state

/// Set when we showed the text.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct ShowedTutorial;

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

/////

use csgrs::polygon::Polygon;
use csgrs::vertex::Vertex;
use csgrs::mesh::Mesh as CsgMesh;

#[allow(unused)]
pub(crate) fn from_bevy_mesh(mesh: &Mesh) -> CsgMesh<()> {
    use csgrs::float_types::parry3d::na::Point3;
    use csgrs::float_types::parry3d::na::Vector3;
    use csgrs::float_types::Real;
    let mut polys = vec![];
    let point_for = |v: Vec3| -> Point3<Real> {
        Point3::new(v.x as _, v.y as _, v.z as _)
    };
    let vec_for = |v: Vec3| -> Vector3<Real> {
        Vector3::new(v.x as _, v.y as _, v.z as _)
    };
    for tri in mesh.triangles().unwrap() {
        let norm = triangle_normal(tri.vertices[0].into(), tri.vertices[1].into(), tri.vertices[2].into());
        let normal = vec_for(norm.into());
        let v0 = Vertex::new(point_for(tri.vertices[0]), normal);
        let v1 = Vertex::new(point_for(tri.vertices[1]), normal);
        let v2 = Vertex::new(point_for(tri.vertices[2]), normal);
        polys.push(Polygon::new(vec![v0, v1, v2], None));
    }
    CsgMesh::from_polygons(&polys, None)
}

fn on_scene_ready(
    ready: On<SceneInstanceReady>,
    children_q: Query<&Children>,
    meshes_q: Query<&Mesh3d>,
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

    let transform_pt = |ptarr: [f32; 3]| -> [f32; 3] {
        // Vec3::from_array(ptarr)
        ptarr
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

            pos.push(transform_pt(pos0));
            pos.push(transform_pt(pos1));
            pos.push(transform_pt(pos2));

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

// fn update_bone_colliders(
//     // newly_sized_meshes: Query<(Entity, &Aabb, &GlobalTransform), (With<Mesh3d>, Added<Aabb>)>,
//     bone_head_collider_q: Query<(Entity, &BoneCollider, &GlobalTransform)>,
//     bone_tail_collider_q: Query<(Entity, &GlobalTransform), With<BoneColliderTail>>,
//     children: Query<&Children>,
//     parent_q: Query<&ChildOf>,
//     bodies_q: Query<Entity, With<RigidBody>>,
//     mut commands: Commands,
// ) {
//     // leg_r: BoneInfo / RigidBody
//     // children:
//     // -- leg head / BoneCollider
//     // -- leg tail / BoneColliderTail

//     for body_ent in bodies_q.iter() {
//         let Ok(parent) = parent_q.get(body_ent) else { continue };
//         let parent = parent.0;

//         // Look for siblings.
//         let Ok(kids) = children.get(parent) else { continue };

//         let mut head_info = None;
//         let mut tail_info = None;

//         for kid in kids {
//             if let Ok(bone) = bone_head_collider_q.get(*kid) {
//                 head_info = Some(bone.clone());
//             }
//             if let Ok(bone) = bone_tail_collider_q.get(*kid) {
//                 tail_info = Some(bone.clone());
//             }
//         }

//         if let Some((head, head_bone, head_gxfrm)) = head_info
//         && let Some((tail, tail_gxfrm)) = tail_info {
//             // See which (if any) edited meshes matches this.
//             if !newly_sized_meshes.contains(head) && !newly_sized_meshes.contains(tail) {
//                 continue
//             }
//             dbg!(head, tail);

//             // The bone colliders are children of bone meshes, and then of the colliders.

//             if bone_head_collider_q.contains(parent) {
//                 // Now look for grandparent with RigidBody.
//                 let Ok(grandparent) = parent_q.get(parent) else { continue };
//                 let grandparent = grandparent.0;
//                 if bodies_q.contains(grandparent) {
//                     // let rot = gxfrm.rotation().angle_between(Quat::from_axis_angle(Vec3::Y, 0.));
//                     // dbg!(rot);
//                     // let center = aabb.center.to_vec3();
//                     // let ext = aabb.half_extents.to_vec3();
//                     // let rot_inv = gxfrm.rotation().inverse();
//                     // let head = center + rot_inv * Vec3::new(0., -ext.y * 2., 0.);
//                     // let tail = center + rot_inv * Vec3::new(0., ext.y * 2., 0.);
//                     let head = head_gxfrm.translation();
//                     let tail = tail_gxfrm.translation();
//                     commands.entity(grandparent).insert((
//                         Collider::capsule_endpoints(0.01, head, tail),
//                     ));
//                 }
//             }
//         }
//     }
// }

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

fn added_player_start(q: Query<&Transform, Added<PlayerStart>>) -> bool {
    let flag = q.iter().next().is_some();
    flag
}

pub(crate) fn spawn_player_on_start(world: &mut World) {
    // Make the player collision model and Player
    let player_ent = spawn_player(world, Uuid::default());

    // Move to start position/orientation.
    let mut start_q = world.query_filtered::<&Transform, With<PlayerStart>>();
    let Some(xfrm) = start_q.iter(world).next() else {
        log::error!("no PlayerStart");
        return;
    };
    drop(start_q);
    let xfrm = xfrm.clone();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);

    // Put and orient the new Player where the PlayerStart is.
    commands.entity(player_ent).insert((
        PlayerLook { rotation: xfrm.rotation, .. default() },
        xfrm
    ));

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
    mut score_q: Query<&mut Text, (With<ScoreArea>, Without<GameStatusArea>)>,
    mut status_q: Query<&mut Text, (With<GameStatusArea>, Without<ScoreArea>)>,
) {
    setup_level(commands.reborrow(), &level_list, &level_index);

    let level = &level_list.0[level_index.0];
    log::info!("Entering level {}", level.label);

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

    score_q.single_mut().unwrap().clear();
    status_q.single_mut().unwrap().clear();
}

pub(crate) fn despawn_level(
    mut commands: Commands,
    sounds_q: Query<Entity, With<SamplePlayer>>,
    spawned_q: Query<Entity, With<Spawned>>,
    player_q: Query<Entity, With<Player>>,
) {
    for ent in sounds_q.iter() {
        commands.entity(ent).try_despawn();
    }
    for ent in spawned_q.iter() {
        commands.entity(ent).try_despawn();
    }
    for ent in player_q.iter() {
        commands.entity(ent).try_despawn();
    }
}
//
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

fn start_skybox_setup(
    mut commands: Commands,
    // world_camera_q: Query<Entity, (With<Camera3d>, With<WorldCamera>)>,
    // skybox_q: Query<&SkyboxSelection, With<LevelRoot>>,
    // skyboxes: Res<SkyboxAssets>,
) {
    // let cam = world_camera_q.single().unwrap();

    // let (brightness, skybox) = (light_consts::lux::CLEAR_SUNRISE, skyboxes.pure_sky.clone());
    // // match selection {
    // //     SkyboxSelection::Space => (100.0, skyboxes.star_map.clone()),
    // //     SkyboxSelection::Farm => (light_consts::lux::CLEAR_SUNRISE, skyboxes.pure_sky.clone()),
    // //     SkyboxSelection::Teeth => (light_consts::lux::LIVING_ROOM, skyboxes.mouth_sky.clone()),
    // //     SkyboxSelection::Station => (light_consts::lux::LIVING_ROOM, skyboxes.station_sky.clone()),
    // // };
    // let with_reflection_probe = Some((cam, 100.0));
    // // let with_reflection_probe = None;
    // commands.entity(cam).insert(SkyboxModel {
    //     skybox: Skybox {
    //         image: skybox,
    //         brightness,
    //         ..default()
    //     },
    //     xfrm: SkyboxTransform::From1_0_2f_3f_4_5,
    //     with_reflection_probe,
    //     enabled: true, //state.show_skybox,
    // });


    // commands.insert_resource(SkyboxSetup {
    //     waiting_skybox: true,
    //     waiting_reflections: false,
    // });
    // commands.set_state(LevelState::Configuring);
    // // } else {
    // // }
    commands.set_state(LevelState::Playing);
}

fn show_instructions(
    mut commands: Commands,
    showed: Option<Res<ShowedTutorial>>,
    fonts: Res<CommonGuiAssets>,
    instructions_q: Single<Entity, With<InstructionsArea>>,
) {
    if showed.is_some() {
        return;
    }

    commands.insert_resource(ShowedTutorial);

    let mut text_ent = Entity::PLACEHOLDER;

    commands.entity(*instructions_q).insert(Visibility::Inherited)  // show
    .with_children(|builder| {
        text_ent = builder.spawn((
            DespawnOnExit(GameplayState::Playing),
            Text::new("",
            ),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
            TextFont {
                font: fonts.std_ui.clone(),
                font_size: 32.0,
                .. default()
            },
            TextColor(Color::WHITE.with_alpha(0.5)),
            TextShadow {
                offset: Vec2::splat(2.),
                color: Color::linear_rgba(0., 0., 0., 0.0),
            },
        )).id();
    });

    // Fade in and out.

    let color_tween = Tween::new(
        EaseMethod::EaseFunction(EaseFunction::CubicOut),
        Duration::from_secs_f32(3.0),
        TextColorLens {
            start: Color::WHITE.with_alpha(0.0),
            end: Color::WHITE.with_alpha(1.0),
        }
    )
    .with_repeat(2, bevy_tweening::RepeatStrategy::MirroredRepeat);

    let shadow_tween = Tween::new(
        EaseMethod::EaseFunction(EaseFunction::CubicOut),
        Duration::from_secs_f32(3.0),
        TextShadowColorLens {
            start: Color::linear_rgba(0., 0., 0., 0.0),
            end: Color::linear_rgba(0., 0., 0., 1.0),
        }
    )
    .with_repeat(2, bevy_tweening::RepeatStrategy::MirroredRepeat);

    commands.entity(text_ent).try_insert((
        DespawnOnExit(GameplayState::Playing),
        TweenAnim::new(color_tween).with_destroy_on_completed(true),

        // Add another TweenAnim.
        children![(
            TweenAnim::new(shadow_tween).with_destroy_on_completed(true),
            AnimTarget::component::<TextShadow>(text_ent),
        )]
    ));
}

pub(crate) fn advance_level(
    mut commands: Commands,
    // spawned_q: Query<Entity, With<Spawned>>,
) {
    // for ent in spawned_q.iter() {
    //     commands.entity(ent).try_despawn();
    // }
    commands.set_state(OverlayState::Loading);
    commands.set_state(GameplayState::Setup);
}

fn update_current_score(
    mut commands: Commands,
    level_state: Res<State<LevelState>>,
    score: Option<Res<CurrentScore>>,
    mut score_q: Single<(&mut Text, &mut TextColor), With<ScoreArea>>,
    // goal_q: Query<&ScoreGoal, With<LevelRoot>>,
) {
    // let Ok(goal) = goal_q.single() else {
    //     if *level_state == LevelState::LoadingSkybox {
    //         // This is allowable, but report once just in case.
    //         log::warn!("missing or too many LevelRoot + ScoreGoal");
    //     };
    //     return;
    // };

    let (ref mut text, ref mut color) = *score_q;
    // if let Some(score) = score {
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
}

fn won_level(
    mut commands: Commands,
    mut score_q: Single<(&mut Text, &mut TextColor), With<GameStatusArea>>,
) {
    let (ref mut text, ref mut color) = *score_q;
    text.0 = "Passed!".to_string();
    color.0 = Color::Srgba(tailwind::LIME_300);

    commands.insert_resource(AutoEndLevelTimer(Timer::new(Duration::from_secs(END_LEVEL_DELAY_SECS), TimerMode::Once)));
}

fn lost_level(
    mut commands: Commands,
    mut score_q: Single<(&mut Text, &mut TextColor), With<GameStatusArea>>,
) {
    let (ref mut text, ref mut color) = *score_q;
    text.0 = "Failed...\nTry again!".to_string();
    color.0 = Color::Srgba(tailwind::RED_700);

    commands.insert_resource(AutoEndLevelTimer(Timer::new(Duration::from_secs(END_LEVEL_DELAY_SECS), TimerMode::Once)));
}

fn check_won_level(
    mut commands: Commands,
    mut end_timer: ResMut<AutoEndLevelTimer>,
    time: Res<Time<Physics>>,
    level_index: ResMut<LevelIndex>,
    level_list: Res<LevelList>,
) {
    if !end_timer.0.tick(time.delta()).is_finished() {
        return;
    }

    let next_index = level_index.0 + 1;
    if next_index >= level_list.0.len() {
        commands.set_state(ProgramState::Completed);
        commands.set_state(LevelState::Initializing);
        commands.set_state(GameplayState::Done);
        commands.set_state(OverlayState::GameOverScreen);

        // Restart next time.
        commands.insert_resource(LevelIndex(0));
    } else {
        commands.insert_resource(LevelIndex(next_index));
        commands.set_state(LevelState::Advance);
    }
}

fn check_lost_level(
    mut commands: Commands,
    mut end_timer: ResMut<AutoEndLevelTimer>,
    time: Res<Time<Physics>>,
) {
    if !end_timer.0.tick(time.delta()).is_finished() {
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
    hand_q: Single<Entity, With<HandStatusArea>>,
    assets: Res<CommonGuiAssets>,
) {
    commands.entity(*hand_q)
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
}

fn remove_power_bar(
    mut commands: Commands,
    child_q: Query<&Children>,
    hand_q: Single<Entity, With<HandStatusArea>>,
) {
    let ent = *hand_q;
    for kid in child_q.iter_descendants(ent) {
        commands.entity(kid).try_despawn();
    }
}

fn update_power_bar(
    mut alpha_q: Single<&mut UiNodeAlpha, With<HandStatusArea>>,
    fire_power: Res<FirePower>,
) {
    if fire_power.is_changed() {
        alpha_q.0 = (**fire_power / 50.0).clamp(0.0, 1.0);
    }
}
