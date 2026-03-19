use std::time::Duration;

use crate::game::*;

use bevy_mod_outline::{OutlineMode, OutlinePlugin, OutlineVolume};
use bevy_seedling::sample::PlaybackSettings;
use bevy_seedling::prelude::*;

use avian3d::prelude::*;
use bevy::prelude::*;
use rand::RngExt as _;
use rand::seq::IndexedRandom as _;

#[cfg(feature = "input_lim")]
use leafwing_input_manager::prelude::*;
#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(OutlinePlugin)
            .add_message::<FireProjectile>()
            .add_message::<ChangeHighlightedItem>()
            .add_systems(
                FixedUpdate,
                (
                    play_player_out_of_bounds,
                )
                .before(TransformSystems::Propagate)
                .after(PhysicsSystems::Writeback)
                .run_if(resource_exists::<CurrentScore>)
                .run_if(not(is_user_paused))
                .run_if(in_state(LevelState::Playing))
                .run_if(in_state(ProgramState::InGame)),
            )

            .init_resource::<GrabbingForce>()
            .init_resource::<CountAccumulator<GrabCycle>>()

            .add_systems(
                FixedUpdate,
                (
                    check_actions,
                    highlight_raycast,
                    handle_fire,
                )
                    .run_if(not(is_paused))
                    .run_if(not(is_in_menu))
                    .run_if(is_level_active)
                    .run_if(not(debug_gui_wants_direct_input))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )

            .add_systems(
                FixedUpdate,
                (
                    cycle_targetables,
                    update_highlight_ui
                        .run_if(resource_changed::<CrosshairTargets>),
                    update_highlight_ui,
                ).chain()
                    .run_if(not(resource_exists::<GrabbedItem>))
                    .run_if(not(is_paused))
                    .run_if(not(is_in_menu))
                    .run_if(is_level_active)
                    .run_if(not(debug_gui_wants_direct_input))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )
        ;

        #[cfg(feature = "input_bei")]
        app.add_systems(
            FixedUpdate,
            check_grab_actions
            .run_if(not(is_in_menu))
            .run_if(is_level_active)
            .run_if(not(is_paused))
            .run_if(not(debug_gui_wants_direct_input))
            .run_if(in_state(ProgramState::InGame))
        );
    }
}

struct GrabCycle;

/// Currently grabbed thing and its transform
/// (Resource only defined if so).
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource, Clone)]
#[type_path = "game"]
pub struct GrabbedItem{
    pub entity: Entity,
    pub orig_offset: Vec3,
    // pub orig_xfrm: GlobalTransform,
    pub distance: f32,
    pub orig_axes: LockedAxes,
    // Movement from original location to un-stick item.
    movement: f32,
    // Movement from original location to un-stick item.
    speed: f32,
}

/// Force that a grabbed object will be moved.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
#[type_path = "game"]
pub struct GrabbingForce(pub f32);

impl Default for GrabbingForce {
    fn default() -> Self {
        Self(25.0)
    }
}

// /// Previously highlighted thing (only defined if so).
// #[derive(Resource, Reflect, Debug)]
// #[reflect(Resource)]
// #[type_path = "game"]
// pub struct LastHighlightedItem(pub Entity);

// /// Currently highlighted thing and offset (only defined if so).
// #[derive(Resource, Reflect, Debug)]
// #[reflect(Resource)]
// #[type_path = "game"]
// pub struct HighlightedItem(pub Entity);

pub(crate) fn play_player_out_of_bounds(
    mut commands: Commands,
    mut reader: MessageReader<HitDeathboxMessage>,
    fx: Res<CommonFxAssets>,
) {
    let mut rng = rand::rng();
    for hit in reader.read() {
        if let HitDeathboxMessage::Player(_) = hit {
            commands.spawn((
                UiSfx,
                SamplePlayer::new(
                    (*[&fx.swoosh]
                        .choose(&mut rng)
                        .unwrap())
                    .clone(),
                ),
                PlaybackSettings {
                    speed: rng.random_range(0.9..1.1),
                    ..default()
                },
                VolumeNode::from_linear(rng.random_range(0.85..1.0)),
            ));
        }
    }
}

#[derive(Message, Debug, Clone)]
struct FireProjectile {
    pub look_xfrm: Transform,
    pub power: f32,
}

#[cfg(feature = "input_lim")]
fn check_actions(
    mut commands: Commands,
    actions: Res<ActionState<UserAction>>,
    player_q: Query<(Entity, &Transform, &ColliderAabb), With<Player>>,
    player_look_q: Query<&PlayerLook>,
    mut fire_power: Local<f32>,
) {
    // Only one player...
    let Ok((player, player_xfrm, aabb)) = player_q.single() else {
        log::error!("no single Player");
        return;
    };
    let Ok(look) = player_look_q.get(player) else {
        log::error!("no PlayerLook");
        return;
    };

    if actions.just_pressed(&UserAction::Fire) {
        *fire_power = 0.1;
    }
    else if actions.pressed(&UserAction::Fire) {
        *fire_power = (*fire_power * 1.25).min(50.0);
    }
    if actions.just_released(&UserAction::Fire) && *fire_power > 0. {
        // Fire something.
        commands.write_message(FireProjectile {
            look_xfrm: Transform::from_translation(player_xfrm.translation).with_rotation(look.rotation),
            power: *fire_power,
        });

        *fire_power = 0.;
    }
}

#[cfg(feature = "input_bei")]
fn check_actions(
    mut commands: Commands,

    fire_events: Query<&ActionEvents, (With<Action<actions::Firing>>, With<PlayerAction>)>,

    player_q: Query<(Entity, &Transform, &ColliderAabb), With<Player>>,
    player_look_q: Query<&PlayerLook>,

    mut fire_power: Local<f32>,

    crosshair_targets: Res<CrosshairTargets>,
    cycle_action_q: Query<(&ActionEvents, &Action<actions::CycleExtendGrab>), With<PlayerAction>>,
    mut cycle_ctr: ResMut<CountAccumulator<GrabCycle>>,
) {
    // Only one player...
    let Ok((player, player_xfrm, aabb)) = player_q.single() else {
        log::error!("no single Player");
        return;
    };
    let Ok(look) = player_look_q.get(player) else {
        log::error!("no PlayerLook");
        return;
    };

    let eyes = player_eyes(player_xfrm, aabb, look);
    let position = player_gun(&look.rotation, eyes);

    let fire = fire_events.iter().next().unwrap();
    if fire.contains(ActionEvents::START) {
        *fire_power = 0.1;
    }
    else if fire.contains(ActionEvents::FIRE) {
        *fire_power = (*fire_power * 1.25).min(50.0);
    }
    else if fire.contains(ActionEvents::COMPLETE) && *fire_power > 0. {
        // Fire something.
        commands.write_message(FireProjectile {
            look_xfrm: Transform::from_translation(position).with_rotation(look.rotation),
            power: *fire_power,
        });

        *fire_power = 0.;
    }

    if let Some((cycle_events, cycle_action)) = cycle_action_q.iter().next() {
        if cycle_events.contains(ActionEvents::START) {
            cycle_ctr.reset();
        }

        if let Some(dir) = cycle_ctr.add_and_test(**cycle_action)
        && !crosshair_targets.targets.is_empty() {
            commands.write_message(ChangeHighlightedItem(dir as isize));
        }

        if cycle_events.contains(ActionEvents::COMPLETE) {
            cycle_ctr.reset();
        }
    }
}

#[derive(Message, Debug)]
struct ChangeHighlightedItem(isize);

/// Update the [Highlighted] item if [CrosshairTargets] changes or the
/// user wants to change the item.
fn cycle_targetables(
    mut commands: Commands,

    mut reader: MessageReader<ChangeHighlightedItem>,

    hilit_q: Query<Entity, (With<Spawned>, With<Highlighted>)>,
    // highlighted_opt: Option<Res<HighlightedItem>>,
    mut crosshair_targets: ResMut<CrosshairTargets>,
) {
    // What was marked Highlighted last frame?
    let old_items = hilit_q.iter().collect::<Vec<_>>();

    let mut first_exist = None;

    // Remove any that are no longer in the crosshair
    // and remember the first candidate that still is.
    for ent in &old_items {
        if crosshair_targets.targets.contains(ent) {
            if first_exist.is_none() {
                first_exist = Some(*ent)
            }
        } else {
            commands.entity(*ent).try_remove::<Highlighted>();
        }
    }

    // See where we index now into the crosshair list.
    let mut new_index = if let Some(first) = &first_exist {
        crosshair_targets.targets.iter().position(|e| *e == *first).expect("we found it above") as isize
    } else {
        0
    };

    // Apply cycle actions.
    for event in reader.read() {
        new_index = new_index.wrapping_add(event.0);
    }

    let new_index = if !crosshair_targets.targets.is_empty() {
        new_index.rem_euclid(crosshair_targets.targets.len() as isize) as usize
    } else {
        0
    };

    // Update resource only if changed.
    if crosshair_targets.index != new_index {
        crosshair_targets.index = new_index;
        if let Some(first) = &first_exist {
            commands.entity(*first).try_remove::<Highlighted>();
        }
    }

    // Highlight the new item, if new.
    if let Some(new_item) = crosshair_targets.targets.get(new_index) {
        if !hilit_q.contains(*new_item) {
            commands.entity(*new_item).try_insert(Highlighted);
        }
    }
}

fn handle_fire(
    mut commands: Commands,
    mut reader: MessageReader<FireProjectile>,

    grabbed_opt: Option<Res<GrabbedItem>>,
    exist_q: Query<Entity>,
    fx: Res<CommonFxAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut any = false;
    for event in reader.read() {
        let xfrm = event.look_xfrm;
        if let Some(grabbed) = &grabbed_opt {
            if exist_q.contains(grabbed.entity) {
                commands.entity(grabbed.entity).insert((
                    LinearVelocity(xfrm.rotation * Vec3::NEG_Z * event.power),
                ));
                release_grab(commands.reborrow(), Some(grabbed.as_ref()));
                any = true;
            }
            commands.remove_resource::<GrabbedItem>();
        } else {

            let mat = materials.add(Color::srgba(0.7, 0.2, 0.2, 1.1));
            let size = Vec3::new(2.0, 0.5, 0.5);
            let mesh = meshes.add(Cuboid::from_size(size));

            commands.spawn(((
                Name::new("BOOM"),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat.clone()),
                xfrm,
                DespawnAfter(Duration::from_secs(120)),
            ), (
                Spawned,
                Projectile,
                CrosshairTargetable,
                CollisionEventsEnabled,
                LinearVelocity(xfrm.rotation * Vec3::NEG_Z * event.power),
                Mass(250.0),
                Friction::new(0.25),
                Restitution::new(0.5),
                SweptCcd::default(),
                RigidBody::Dynamic,
                Collider::cuboid(size.x, size.y, size.z),
            )));
            any = true;
        }
    }

    if any {
        commands.spawn((
            UiSfx,
            SamplePlayer::new(fx.swoosh.clone()),
        ));
    }
}

fn highlight_raycast(
    mut info_q: Single<(&mut Text, &mut TextColor, &mut Visibility), With<InfoArea>>,
    targets_q: Query<Option<&Name>>,
    crosshair_target: Res<CrosshairTargets>,
) {
    let (ref mut text, ref mut color, ref mut visibility) = *info_q;
    if crosshair_target.targets.is_empty() || !dev_tools_enabled() {
        visibility.set_if_neq(Visibility::Hidden);
    } else {
        visibility.set_if_neq(Visibility::Inherited);

        let mut message = "[".to_string();
        let mut started = false;
        let current = crosshair_target.targets.get(crosshair_target.index).cloned();
        for ent in &crosshair_target.targets {
            let Ok(name_opt) = targets_q.get(*ent) else { continue };
            if !started {
                started = true
            } else {
                message += ", "
            }
            if current == Some(*ent) {
                message += "*";
            }
            let segment = if let Some(name) = name_opt {
                format!("{ent}: \"{name}\"")
            } else {
                ent.to_string()
            };
            message += &segment;
        }
        message += "]";
        text.0 = message;
        color.0 = Color::Srgba(tailwind::GRAY_100);
    }
}

/// When [CrosshairTargets] changes, remove/add [Highlighted].
fn update_highlight_ui(
    mut commands: Commands,
    fx: Res<CommonFxAssets>,
    now_hovered_q: Query<Entity, (With<Spawned>, Added<Highlighted>)>,
    was_hovered_q: Query<Entity, (With<OutlineVolume>, With<Spawned>, Without<Highlighted>)>,
) {
    for ent in was_hovered_q.iter() {
        commands.entity(ent).try_remove::<Highlighted>();
        commands.entity(ent).try_remove::<OutlineVolume>();
    }

    let mut any = false;
    for ent in now_hovered_q.iter() {
        commands.entity(ent).try_insert((
            Highlighted,
            OutlineVolume {
                visible: true,
                width: 4.0,
                colour: Color::WHITE.with_alpha(0.5),
            },
            OutlineMode::FloodFlat,
        ));
        any = true;
    }
    if any {
        let mut rng = rand::rng();
        commands.spawn((
            UiSfx,
            SamplePlayer::new(
                (*[&fx.select]
                    .choose(&mut rng)
                    .unwrap())
                .clone(),
            ),
            PlaybackSettings {
                speed: rng.random_range(0.9..1.1),
                ..default()
            },
            VolumeNode::from_linear(rng.random_range(0.85..1.0)),
        ));
    }
}

/// See if the user is grabbing/dragging/ungrabbing something.
#[cfg(feature = "input_bei")]
fn check_grab_actions(
    mut commands: Commands,

    grab_action_q: Query<(&ActionEvents, &ActionTime), (With<Action<actions::ToggleGrab>>, With<PlayerAction>)>,
    extend_action_q: Query<(&ActionEvents, &Action<actions::CycleExtendGrab>), With<PlayerAction>>,

    // highlight_opt: Option<Res<HighlightedItem>>,
    hilit_q: Query<Entity, With<Highlighted>>,
    mut grabbed_opt: Option<ResMut<GrabbedItem>>,
    grabbing_force: Res<GrabbingForce>,

    mut gizmos: Gizmos,
    camera_q: Single<&GlobalTransform, (With<Camera3d>, With<WorldCamera>)>,

    mut raycast: MeshRayCast,
    mut phys_info_q: Query<(&GlobalTransform, &Transform, Option<&LockedAxes>, Forces)>,
) {
    let Some((grab_events, _grab_time)) = grab_action_q.iter().next() else { return };

    let cam_global_xfrm = *camera_q;

    if grab_events.contains(ActionEvents::START) {
        // Try to grab.
        if grabbed_opt.is_none()
        && let Some(highlight) = hilit_q.iter().next()
        && let Ok((item_global_xfrm, _, axes, _)) = phys_info_q.get(highlight) {

            // We can have clicked anywhere on the grabbed object,
            // but later compute grab distance based on the center.
            // Account for that here.
            let cam_pos = cam_global_xfrm.translation();
            let cam_dir = Dir3::new(cam_global_xfrm.rotation() * Vec3::NEG_Z).unwrap_or(Dir3::NEG_Z);
            let cur_pos = item_global_xfrm.translation();
            let hits = raycast.cast_ray(
                Ray3d::new(cam_pos, cam_dir),
                &MeshRayCastSettings::default()
                    .with_filter(&|ent| ent == highlight)
                    .never_early_exit()
            );
            let new_pos = if let Some(hit) = hits.get(0) {
                hit.1.point
            } else {
                cur_pos
            };
            let distance = cam_pos.distance(new_pos);

            commands.insert_resource(GrabbedItem{
                entity: highlight,
                orig_offset: new_pos - cur_pos,
                distance,
                orig_axes: axes.map_or(default(), |a| *a),
                movement: 0.,
                speed: 0.,
            });

            commands.entity(highlight).try_insert((
                Selected,
                OutlineVolume {
                visible: true,
                    width: 4.0,
                    colour: tailwind::LIME_500.into(),
                },
                OutlineMode::FloodFlat,

                // LockedAxes::ALL_LOCKED,
                LockedAxes::ROTATION_LOCKED,
            ));
        }
    } else if grab_events.contains(ActionEvents::FIRE) && let Some(grabbed) = &mut grabbed_opt {
        if let Some((extend_events, extend_action)) = extend_action_q.iter().next() {
            if extend_events.contains(ActionEvents::FIRE) {
                let new_dist = (grabbed.distance + **extend_action).clamp(0.1, 100.0);
                grabbed.distance = new_dist;
            }
        }

        if let Ok((item_global_xfrm, xfrm, _, mut forces)) = phys_info_q.get_mut(grabbed.entity) {
            // Currently grabbing (and moving?)

            // Compute the desired new location, i.e. the current
            // position plus the camera's position + original distance,
            // then apply an impulse to move to that location.
            let cur_pos = item_global_xfrm.translation() + grabbed.orig_offset;

            let cam_pos = cam_global_xfrm.translation();
            let new_pos = cam_pos + cam_global_xfrm.rotation() * Vec3::NEG_Z * grabbed.distance;

            let offset = new_pos - cur_pos;

            let movement = offset.length();
            if movement > 0.01 {
                grabbed.speed = grabbed.speed.max(0.05) * 1.01;
                *forces.linear_velocity_mut() = offset * grabbed.speed * grabbing_force.0;
                *forces.angular_velocity_mut() = default();
                grabbed.movement += movement;
            } else {
                grabbed.speed *= 0.99;
            }

            // Draw axes from all edges.
            gizmos.axes(*xfrm, grabbing_force.0);

            let mut inv_xfrm = xfrm.clone();
            inv_xfrm.rotate_local_x(std::f32::consts::PI);
            gizmos.axes(inv_xfrm, grabbing_force.0);

            let mut inv_xfrm = xfrm.clone();
            inv_xfrm.rotate_local_y(std::f32::consts::PI);
            gizmos.axes(inv_xfrm, grabbing_force.0);

            let mut inv_xfrm = xfrm.clone();
            inv_xfrm.rotate_local_z(std::f32::consts::PI);
            gizmos.axes(inv_xfrm, grabbing_force.0);
        }

    } else if grab_events.contains(ActionEvents::COMPLETE) {
        // Let go.
        let grabbed_opt = grabbed_opt.map(|gi| gi.clone());
        release_grab(commands.reborrow(), grabbed_opt.as_ref());
    }
}

fn release_grab(mut commands: Commands, grabbed_opt: Option<&GrabbedItem>) {
    commands.remove_resource::<GrabbedItem>();
    if let Some(grabbed) = &grabbed_opt {
        let mut ent_commands = commands.entity(grabbed.entity);
        ent_commands.try_remove::<Selected>();
        ent_commands.try_remove::<OutlineVolume>();
        ent_commands.try_remove::<OutlineMode>();
        if grabbed.orig_axes.to_bits() != 0 {
            ent_commands.insert(grabbed.orig_axes);
        } else {
            ent_commands.try_remove::<LockedAxes>();
        }
        ent_commands.try_remove::<Sleeping>();
    }
}
