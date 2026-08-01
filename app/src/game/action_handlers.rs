use rand::RngExt as _;
use rand::seq::IndexedRandom as _;

use bevy_seedling::sample::PlaybackSettings;
use bevy_seedling::prelude::*;
use bevy::prelude::*;

#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;

#[cfg(feature = "input_bei")]
use crate::game::firing::{FireActionState, cancel_projectile, prepare_projectile};
use crate::game::*;

pub struct ActionHandlersPlugin;

impl Plugin for ActionHandlersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                FixedUpdate,
                play_player_out_of_bounds
                .run_if(not(is_paused))
                .run_if(in_state(LevelState::Playing))
                .run_if(in_state(ProgramState::InGame)),
            )

            .add_observer(on_firing_start)
            .add_observer(on_firing_hold)
            .add_observer(on_firing_release)
            .add_observer(on_firing_cancel)
            .add_observer(on_flashlight_toggle)
        ;
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
                VolumeNode::from_linear(rng.random_range(0.25..0.5)),
            ));
        }
    }
}

#[cfg(feature = "input_bei")]
fn on_firing_start(
    _fire: On<Start<actions::Firing>>,
    alt_fire: Single<&TriggerState, (With<Action<actions::AltFiring>>, With<ActionOf<PlayerContext>>)>,
    mut commands: Commands,
    mut fire_state: ResMut<FiringState>,
    limits: Res<FirePowerLimits>,
    grabbed_opt: Option<Res<GrabbedItem>>,
) {
    if !matches!(*alt_fire, TriggerState::None) {
        return
    }
    (*fire_state).start(&limits);
    if grabbed_opt.is_none() {
        commands.run_system_cached(prepare_projectile);
    }
}

#[cfg(feature = "input_bei")]
fn on_firing_hold(
    fire: On<Fire<actions::Firing>>,
    alt_fire: Single<&TriggerState, (With<Action<actions::AltFiring>>, With<ActionOf<PlayerContext>>)>,
    mut fire_state: ResMut<FiringState>,
    limits: Res<FirePowerLimits>,
) {
    if !matches!(*alt_fire, TriggerState::None) {
        return
    }
    (*fire_state).update(fire.fired_secs, &limits);
}

#[cfg(feature = "input_bei")]
fn on_firing_release(
    _fire: On<Complete<actions::Firing>>,
    alt_fire: Single<&TriggerState, (With<Action<actions::AltFiring>>, With<ActionOf<PlayerContext>>)>,
    mut commands: Commands,
    mut params: If<FireActionState>,
    spatial: If<SpatialQuery>,
    meshes: ResMut<Assets<Mesh>>,
    xfrm_q: Query<&Transform>,
) {
    if !matches!(*alt_fire, TriggerState::None) {
        return
    }

    (*params).fire_projectile(commands.reborrow(), &*spatial, meshes, xfrm_q);
}

/// We implement firing cancellation as using Alt fire to cancel it.
#[cfg(feature = "input_bei")]
fn on_firing_cancel(
    _fire: On<Fire<actions::AltFiring>>,
    mut fire_action: Single<
        &mut TriggerState, (
            With<Action<actions::Firing>>,
            With<ActionOf<PlayerContext>>,
        )>,
    mut fire_state: If<ResMut<FiringState>>,
    mut commands: Commands,
) {
    (*fire_state).clear();
    **fire_action = TriggerState::None;

    commands.run_system_cached(cancel_projectile);
}

#[cfg(feature = "input_bei")]
fn on_flashlight_toggle(
    _fire: On<Start<actions::ToggleFlashlight>>,
    camera_q: Query<Entity, With<PlayerCamera>>,
    child_q: Query<&Children>,
    mut flashlight_q: Query<&mut Flashlight>,
) {
    let Ok(camera) = camera_q.single() else {
        warn!("no single PlayerCamera");
        return
    };

    for ent in child_q.iter_descendants(camera) {
        if let Ok(mut flashlight) = flashlight_q.get_mut(ent) {
            flashlight.enabled ^= true;
        }
    }
}
