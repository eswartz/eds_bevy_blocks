use std::time::Duration;

use crate::game::*;

use avian3d::math::AdjustPrecision as _;
use avian3d::math::Scalar;
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
            .insert_resource(FirePowerWindup {
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
pub struct FirePowerWindup {
    pub accel: f32,
    pub start: f32,
    pub max: f32,
}

impl FirePowerWindup {
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
        // Emit sound effect is we're about to be sent to start.
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
    fire_power_windup: Res<FirePowerWindup>,
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
        **fire_power = fire_power_windup.start;
    }
    else if actions.pressed(&UserAction::Fire) {
        **fire_power = fire_power_windup.apply_force(time.delta(), **fire_power);
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

    mut mesh_params: ParamSet<(
        (ResMut<Assets<Mesh>>,),
        (MeshRayCast,)
    )
    >,

    world: Res<WorldMarkerEntity>,

    grabbed_opt: Option<Res<GrabbedItem>>,

    mut fire_power: ResMut<FirePower>,
    fire_power_windup: Res<FirePowerWindup>,

    boom_mass: Res<BoomMass>,
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
            **fire_power = fire_power_windup.start;
        }
        else if fire.contains(ActionEvents::FIRE) {
            **fire_power = fire_power_windup.apply_force(time.delta(), **fire_power);
        }
        else if fire.contains(ActionEvents::COMPLETE) && **fire_power > 0. {
            // Fire something.

            // FIXME: needs to be outside character collider.
            let mut pos = position + look.rotation * Vec3::NEG_Z;

            let ray = Ray3d::new(player_xfrm.translation, look.rotation * Dir3::NEG_Z);
            let mut params = mesh_params.p1();
            let hits = params.0.cast_ray(ray, &MeshRayCastSettings::default()
                .always_early_exit()
                .with_visibility(RayCastVisibility::Visible),
            );
            if let Some(hit) = hits.get(0) {
                // Adjust to world.
                // pos = hit.1.distance;
                pos = position + look.rotation * Vec3::NEG_Z * (hit.1.distance.min(1.0));
            }

            let xfrm = Transform::from_translation(pos).with_rotation(look.rotation);
            let power = **fire_power;

            do_fire(commands.reborrow(), xfrm, power, grabbed_opt, exist_q,
                fx, materials, mesh_params.p0().0, world, &boom_mass, highlighting_mode);

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

    world: Res<WorldMarkerEntity>,

    boom_mass: &BoomMass,
    mut highlighting_mode: ResMut<HighlightingMode>,
) -> bool {
    let vel = xfrm.rotation * Vec3::NEG_Z * power;
    let mut any = false;
    if let Some(grabbed) = &grabbed_opt {
        // Fire the item we are holding, if it still exists.
        if exist_q.contains(grabbed.entity) {
            commands.queue(WakeBody(grabbed.entity));
            commands.entity(grabbed.entity).insert((
                LinearVelocity(vel.adjust_precision()),
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
            ChildOf(world.0),
            Name::new("BOOM"),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            xfrm,
            // DespawnAfter(Duration::from_secs(120)),
        ), (
            Spawned,
            Projectile,
            CrosshairTargetable,
            CollisionEventsEnabled,
            LinearVelocity(vel.adjust_precision()),
            Mass(boom_mass.0),
            Friction::new(0.25),
            Restitution::new(0.5),
            SweptCcd::default(),
            RigidBody::Dynamic,
            Collider::cuboid(size.x as Scalar, size.y as Scalar, size.z as Scalar),
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
