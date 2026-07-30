use rand::RngExt as _;
use rand::seq::IndexedRandom as _;

use bevy_seedling::sample::PlaybackSettings;
use bevy_seedling::prelude::*;
use bevy::prelude::*;

#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;

use crate::game::firing::{FiredObject, fire_projectile};
use crate::game::*;

pub struct ActionHandlersPlugin;

impl Plugin for ActionHandlersPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FiringState>()
            .init_resource::<FiredObject>()
            .insert_resource(FirePowerLimits {
                accel: 1.1,
                max: 50.0,
                start: 0.1,
            })

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
    mut fire_state: ResMut<FiringState>,
    limits: Res<FirePowerLimits>,
) {
    (*fire_state).start(&limits);
}

#[cfg(feature = "input_bei")]
fn on_firing_hold(
    fire: On<Fire<actions::Firing>>,
    mut fire_state: ResMut<FiringState>,
    limits: Res<FirePowerLimits>,
) {
    (*fire_state).update(fire.fired_secs, &limits);
}

#[cfg(feature = "input_bei")]
fn on_firing_release(
    _fire: On<Complete<actions::Firing>>,
    mut commands: Commands,
    mut fire_state: ResMut<FiringState>,
) {
    let power = fire_state.power();
    (*fire_state).clear();

    commands.run_system_cached_with(fire_projectile, power);
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
