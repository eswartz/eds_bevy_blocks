use std::time::Duration;

use crate::game::*;

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
            .add_plugins(HighlightingPlugin)
            .add_plugins(GrabbingPlugin)
            .insert_resource(HighlightingMode::Disabled)
            .init_resource::<FirePower>()
            .insert_resource(FirePowerStats {
                accel: 1.1,
                max: 100.0,
                start: 0.1,
            })

            .add_systems(
                FixedUpdate,
                play_player_out_of_bounds
                .run_if(not(is_user_paused))
                .run_if(in_state(LevelState::Playing))
                .run_if(in_state(ProgramState::InGame)),
            )

            // .add_systems(
            //     FixedUpdate,
            //     (
            //         decay_forces, //.run_if(input_just_pressed(KeyCode::Backslash)),
            //     )
            //     .before(PhysicsSystems::Writeback)
            //     .run_if(not(is_user_paused))
            //     .run_if(in_state(LevelState::Playing))
            //     .run_if(in_state(ProgramState::InGame)),
            // )
            // .add_systems(
            //     FixedUpdate,
            //     decay_physics
            //         .before(PhysicsSystems::StepSimulation)
            //         .run_if(not(is_paused))
            //         .run_if(in_state(LevelState::Playing))
            //         .run_if(in_state(ProgramState::InGame))
            //         // .run_if(is_in_level(ID))
            // )

            .add_systems(
                FixedUpdate,
                (
                    check_actions,
                    report_raycast,
                    // handle_fire,
                )
                    .run_if(not(is_paused))
                    .run_if(not(is_in_menu))
                    .run_if(is_level_active)
                    .run_if(not(debug_gui_wants_direct_input))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )
        ;
    }
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "game"]
pub struct FirePowerSound;

#[derive(Resource, Default, Debug, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub struct FirePower(pub f32);

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub struct FirePowerStats {
    pub accel: f32,
    pub start: f32,
    pub max: f32,
}

impl FirePowerStats {
    pub(crate) fn apply_force(&self, dt: Duration, power: f32) -> f32 {
        let q = (dt.as_secs_f32() * 64.0).min(1.0);
        let mul = 1.0.lerp(self.accel, q);
        (power * mul).min(self.max)
    }
}


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

#[cfg(feature = "input_lim")]
fn check_actions(
    mut commands: Commands,
    actions: Res<ActionState<UserAction>>,
    player_q: Query<(Entity, &Transform, &ColliderAabb), With<Player>>,
    player_look_q: Query<&PlayerLook>,
    mut fire_power: ResMut<FirePower>,
    fire_power_stats: Res<FirePowerStats>,
    time: Res<Time>,
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
        **fire_power = fire_power_stats.start;
    }
    else if actions.pressed(&UserAction::Fire) {
        **fire_power = fire_power_stats.apply_force(time.delta(), **fire_power);
    }
    if actions.just_released(&UserAction::Fire) && **fire_power > 0. {
        // Fire something.
        commands.write_message(FireProjectile {
            look_xfrm: Transform::from_translation(player_xfrm.translation).with_rotation(look.rotation),
            power: **fire_power,
        });

        **fire_power = 0.;
    }
}

#[cfg(feature = "input_bei")]
fn check_actions(
    mut commands: Commands,

    fire_events: Query<&ActionEvents, (With<Action<actions::Firing>>, With<PlayerAction>)>,

    select_events: Query<&ActionEvents, (With<Action<actions::Interact>>, With<PlayerAction>)>,
    mut highlighting_mode: ResMut<HighlightingMode>,

    player_q: Query<(Entity, &Transform, &ColliderAabb), With<Player>>,
    player_look_q: Query<&PlayerLook>,

    exist_q: Query<Entity>,
    fx: Res<CommonFxAssets>,
    materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,

    grabbed_opt: Option<Res<GrabbedItem>>,
    mut fire_power: ResMut<FirePower>,
    fire_power_stats: Res<FirePowerStats>,
    time: Res<Time>,
) {
    if let Ok(select) = select_events.single() {
        if select.contains(ActionEvents::START) {
            *highlighting_mode = (*highlighting_mode).toggle_enabled();
        }
    }

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

    if let Ok(fire) = fire_events.single() {
        if fire.contains(ActionEvents::START) {
            **fire_power = fire_power_stats.start;
        }
        else if fire.contains(ActionEvents::FIRE) {
            **fire_power = fire_power_stats.apply_force(time.delta(), **fire_power);
        }
        else if fire.contains(ActionEvents::COMPLETE) && **fire_power > 0. {
            // Fire something.

            let xfrm = Transform::from_translation(position).with_rotation(look.rotation);
            let power = **fire_power;

            do_fire(commands.reborrow(), xfrm, power, grabbed_opt, exist_q,
                fx, materials, meshes, highlighting_mode);

            **fire_power = 0.;
        }
    }
}

fn do_fire(
    mut commands: Commands,

    xfrm: Transform,
    power: f32,

    grabbed_opt: Option<Res<GrabbedItem>>,

    exist_q: Query<Entity>,
    fx: Res<CommonFxAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,

    mut highlighting_mode: ResMut<HighlightingMode>,
) -> bool {
    let vel = xfrm.rotation * Vec3::NEG_Z * power;
    let mut any = false;
    if let Some(grabbed) = &grabbed_opt {
        // Fire the item we are holding, if it still exists.
        if exist_q.contains(grabbed.entity) {
            commands.queue(WakeBody(grabbed.entity));
            commands.entity(grabbed.entity).insert((
                LinearVelocity(vel),
            ));
            commands.write_message(GrabbingCommand::ReleaseItems);
            any = true;
        } else {
            commands.write_message(GrabbingCommand::CancelGrabItems);
        }
    } else {
        // Fire a new item.
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
            LinearVelocity(vel),
            Mass(250.0),
            Friction::new(0.25),
            Restitution::new(0.5),
            SweptCcd::default(),
            RigidBody::Dynamic,
            Collider::cuboid(size.x, size.y, size.z),
        )));
        any = true;
    }

    if any {
        *highlighting_mode = HighlightingMode::Enabled;

        commands.spawn((
            UiSfx,
            SamplePlayer::new(fx.swoosh.clone()),
        ));
    }

    any
}

fn report_raycast(
    mut info_q: Single<(&mut Text, &mut TextColor, &mut Visibility), With<InfoArea>>,
    highlighting_mode: Res<HighlightingMode>,
    crosshair_target: Res<CrosshairTargets>,
    names_q: Query<Option<&Name>>,
) {
    if !dev_tools_enabled() {
        return
    }

    let (ref mut text, ref mut color, ref mut visibility) = *info_q;
    if highlighting_mode.is_enabled()
    && let Some(message) = report_crosshair_targets(&crosshair_target, &names_q) {
        visibility.set_if_neq(Visibility::Inherited);
        text.0 = message;
        color.0 = Color::Srgba(tailwind::GRAY_100);
    } else {
        visibility.set_if_neq(Visibility::Hidden);
    }
}

fn decay_forces(
    mut forces_q: Query<Forces, (With<Spawned>, With<RigidBody>)>,
) {
    forces_q.par_iter_mut().for_each(|mut forces| {
        let lsq = forces.linear_velocity().length_squared();
        let asq = forces.angular_velocity().length_squared();
        if (lsq > 0. && lsq < 0.1) || (asq > 0. && asq < 0.1) {
            let mut nw = forces.non_waking();
            *nw.linear_velocity_mut() = default();
            *nw.angular_velocity_mut() = default();
            nw.reset_accumulated_linear_acceleration();
            nw.reset_accumulated_angular_acceleration();
        }
    });
}

// const LIVE_LIN_DAMP: f32 = 0.125;
// const LIVE_ANG_DAMP: f32 = 0.125;
// const SLEEP_LIN_DAMP: f32 = 0.95;
// const SLEEP_ANG_DAMP: f32 = 0.95;

// const LIVE_VEL_LEN_SQ: f32 = 0.5;
// const LIVE_ANG_LEN_SQ: f32 = 0.125;
// const SLEEP_VEL_LEN_SQ: f32 = 0.125;
// const SLEEP_ANG_LEN_SQ: f32 = 0.01;

// fn decay_physics(
//     mut coll_q: Query<
//         (Forces, &mut LinearDamping, &mut AngularDamping),
//         (With<Spawned>, With<Cube>, Without<Sleeping>)
//     >) {

//     coll_q.par_iter_mut().for_each(|(mut forces, mut lin_damp, mut ang_damp)| {
//         let vel_sq = forces.linear_velocity().length_squared();
//         let ang_sq = forces.angular_velocity().length_squared();

//         if vel_sq >= LIVE_VEL_LEN_SQ {
//             // Turn down damping when moving.
//             if lin_damp.0 > LIVE_LIN_DAMP {
//                 lin_damp.0 = LIVE_LIN_DAMP;
//             }
//         } else if vel_sq < SLEEP_VEL_LEN_SQ {
//             if lin_damp.0 < SLEEP_LIN_DAMP {
//                 let mut nw = forces.non_waking();
//                 *nw.linear_velocity_mut() = default();
//                 nw.reset_accumulated_linear_acceleration();
//                 lin_damp.0 = SLEEP_LIN_DAMP;
//             }
//         }

//         if ang_sq >= LIVE_ANG_LEN_SQ {
//             // Turn down damping when rotating.
//             if ang_damp.0 > LIVE_ANG_DAMP {
//                 ang_damp.0 = LIVE_ANG_DAMP;
//             }
//         } else if ang_sq < SLEEP_ANG_LEN_SQ {
//             if ang_damp.0 < SLEEP_ANG_DAMP {
//                 let mut nw = forces.non_waking();
//                 *nw.angular_velocity_mut() = default();
//                 nw.reset_accumulated_angular_acceleration();
//                 ang_damp.0 = SLEEP_ANG_DAMP;
//             }
//         }
//     });
// }
