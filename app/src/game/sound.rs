use crate::game::{Cube, Floor};

use avian3d::prelude::{Collisions, LinearVelocity};
use bevy_seedling::prelude::{SpatialBasicNode, VolumeNode};
use bevy_seedling::sample::SamplePlayer;
use bevy_seedling::sample_effects;
use eds_bevy_common::*;
use bevy::prelude::*;

use rand::RngExt as _;
use rand::seq::IndexedRandom as _;

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update,
                spawn_noise_on_collision
                .run_if(not(is_paused))
                .run_if(in_state(LevelState::Playing))
                .run_if(in_state(GameplayState::Playing))
            )
        ;
    }
}

fn spawn_noise_on_collision(
    collisions: Collisions,
    fx: Res<CommonFxAssets>,
    xfrm_vel_q: Query<(&GlobalTransform, &LinearVelocity)>,
    projectile_q: Query<&Projectile>,
    cube_q: Query<&Cube>,
    floor_q: Query<&Floor>,
    player_q: Query<&Player>,
    mut commands: Commands,
) {
    let mut rng = rand::rng();
    let mut added = 0;

    for event in collisions.iter() {
        if event.collision_ended() {
            continue
        }
        if !event.is_touching() {
            continue
        }
        let cube_cube = cube_q.contains(event.collider1) && cube_q.contains(event.collider2);
        let floor = floor_q.contains(event.collider1) || floor_q.contains(event.collider2);
        if floor && (player_q.contains(event.collider1) || player_q.contains(event.collider2)) {
            continue
        }

        let (src, target) = if cube_cube {
            (event.collider1, event.collider2)
        } else {
            if projectile_q.contains(event.collider2) {
                (event.collider1, event.collider2)
            } else if projectile_q.contains(event.collider1) || floor {
                (event.collider2, event.collider1)
            } else {
                continue
            }
        };

        if let Ok((xfrm, vel)) = xfrm_vel_q.get(target)
        {
            let src_vel = xfrm_vel_q.get(src).map_or_else(|_| &Vec3::ZERO, |(_, src_vel)| src_vel);

            let vel_length = vel.0.distance(*src_vel);
            if vel_length < 1.0 {
                // They're moving slowly relative to each other,
                // filter out
                continue
            }

            // Distinguish between "small" impulses and "large" impulses
            // using the log scale.
            let mag = (event.total_normal_impulse_magnitude() + 0.01).log10();
            let silent = mag < 0.1;
            if silent {
                // Too weak to make a noise.
                continue
            }
            let mag = (mag / 8.0).clamp(0.0, 1.0);

            let effect = if floor {
            (*[
                &fx.brush1a,
                &fx.brush1b,
                &fx.brush1c,
                &fx.brush1d,
                &fx.brush1e,
                &fx.brush1f,
                ]
                .choose(&mut rng)
                .unwrap())
                .clone()
            } else if cube_cube {
                (*[
                    &fx.wood1a,
                    &fx.wood1b,
                    &fx.wood1c,
                    &fx.wood1d,

                    &fx.snap1b,
                    &fx.snap1c,
                    &fx.snap1d,
                    &fx.snap1e,
                    &fx.snap1f,
                    &fx.snap1g,
                    &fx.snap1h,
                    &fx.snap1i,
                    &fx.snap1j,
                    &fx.snap1k,
                    &fx.snap1l,
                    &fx.snap1m,
                    &fx.snap1n,
                    &fx.snap1o,
                    &fx.snap1p,
                    &fx.snap1q,
                    &fx.snap1r,

                    ]
                    .choose(&mut rng)
                    .unwrap())
                    .clone()
            } else if vel_length < 5.0 {
                (*[&fx.bump0a, &fx.bump0b, &fx.bump0c]
                    .choose(&mut rng)
                    .unwrap())
                    .clone()
            } else {
                (*[&fx.bump2, &fx.bump3]
                    .choose(&mut rng)
                    .unwrap())
                    .clone()
            };

            commands.spawn((
                Sfx,
                SamplePlayer::new(
                    effect,
                ),
                xfrm.clone(),
                sample_effects![SpatialBasicNode::default()],
                PlaybackSettings {
                    speed: rng.random_range(0.9..1.1),
                    ..default()
                },
                VolumeNode::from_linear(mag * rng.random_range(0.25..0.5)),
            ));
            // commands.spawn((
            //     UiSfx,
            //     SamplePlayer::new(
            //         effect,
            //     ),
            //     // xfrm.clone(),
            //     // sample_effects![SpatialBasicNode::default()],
            //     PlaybackSettings {
            //         speed: rng.random_range(0.5..1.5),
            //         ..default()
            //     },
            //     VolumeNode::from_linear(rng.random_range(0.01..0.25)),
            // ));

            added += 1;
            if added > 4 {
                break
            }
        }
    }
}
