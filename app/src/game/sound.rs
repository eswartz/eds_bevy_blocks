use avian3d::{dynamics::rigid_body::{AngularVelocity, mass_properties::components::Mass}, prelude::{Collisions, LinearVelocity}};
use bevy_seedling::{firewheel::Volume, prelude::*, sample::{AudioSample, RandomPitch, SamplePlayer}};
use eds_bevy_common::*;
use bevy::prelude::*;

use rand::{RngExt as _, seq::IndexedRandom as _};

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update,
                (
                    spawn_noise_on_collision,
                )
                    .run_if(not(is_paused))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(GameplayState::Playing))
            )
        ;
    }
}

fn spawn_noise_on_collision(
    surf_mat_q: Query<&SurfaceMaterial>,

    collisions: Collisions,
    fx: Res<CommonFxAssets>,
    phys_info_q: Query<(&GlobalTransform, &LinearVelocity, &AngularVelocity, &Mass)>,
    listener_q: Query<&GlobalTransform, With<SpatialListener3D>>,
    player_q: Query<&Player>,
    parent_q: Query<&ChildOf>,
    paused: Res<PhysicsPaused>,

    mut footstep_dist: Local<f32>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if **paused {
        return
    }

    let mut rng = rand::rng();
    let mut added = 0;

    let listener_xfrm = listener_q.iter().next().cloned().unwrap_or_default();

    for event in collisions.iter() {
        if !event.collision_started() && !event.is_touching() {
            continue
        }

        let player_a = player_q.contains(event.collider1);
        let player_b = player_q.contains(event.collider2);
        let one_is_player = player_a || player_b;

        let has_mat_a = surf_mat_q.contains(event.collider1);
        let has_mat_b = surf_mat_q.contains(event.collider2);
        let one_has_mat = has_mat_a || has_mat_b;

        let get_mat = |mut ent| {
            loop {
                if let Ok(mat) = surf_mat_q.get(ent) { return *mat };
                let Ok(parent) = parent_q.get(ent) else { return default() };
                ent = parent.0;
            }
        };

        let phys_mat_a = get_mat(event.collider1);
        let phys_mat_b = get_mat(event.collider2);

        let (src, target) =
            if has_mat_b || player_b {
                (event.collider1, event.collider2)
            } else if has_mat_a || player_a {
                (event.collider2, event.collider1)
            } else {
                continue
            }
        ;

        if let Ok((xfrm, vel, ang_vel, mass)) = phys_info_q.get(target)
        {
            let (src_vel, src_ang_vel) = phys_info_q
                .get(src)
                .map_or_else(
                    |_| (&Vec3::ZERO, &Vec3::ZERO),
                    |(_, src_vel, src_ang, _)| (&*src_vel, &*src_ang));

            let rel_vel = *src_vel - vel.0;
            let vel_length = rel_vel.length();
            let rel_ang_abs = src_ang_vel.abs() - ang_vel.0.abs();
            let ang_length = rel_ang_abs.length() /* rad */ * std::f32::consts::PI * 0.125 /* m */;
            if vel_length + ang_length < 1.0 {
                // They're moving slowly relative to each other, ignore
                continue
            }

            // Distinguish between "small" impulses and "large" impulses using the log scale.
            let impulse_log = (event.total_normal_impulse_magnitude() + 0.01).log10();
            let silent = impulse_log < 0.05;
            if silent {
                // Too weak to make a noise.
                continue
            }

            // Distinguish impact from sliding.
            let sliding = event.is_touching() && !event.manifolds.is_empty() && {
                let normal = event.manifolds[0].normal;

                let vel_rel_n = rel_vel.dot(normal);
                let vel_rel_t = rel_vel - rel_vel.dot(normal) * normal;
                let sliding_speed = vel_rel_t.length();
                let max_slide_speed = if one_is_player { 8.0 } else { 2.0 };
                let sliding = vel_rel_n.abs() < 0.1 && sliding_speed > max_slide_speed;
                sliding
            };

            let target_entity: Entity;
            let sample: Handle<AudioSample>;
            let vol_range: core::ops::Range<f32>;
            let speed_range: core::ops::Range<f32>;
            if one_is_player {
                const FOOTFALL_TIMES_SAMPLE_DIST: f32 = 3.0;
                let dist = vel_length * time.delta_secs();
                *footstep_dist += dist;
                if *footstep_dist < 0.0 {
                    continue
                }

                *footstep_dist -= FOOTFALL_TIMES_SAMPLE_DIST;
                if !sliding {
                    let (ent, phys_mat) = if player_a { (event.collider2, phys_mat_b) } else { (event.collider1, phys_mat_a) };
                    target_entity = ent;
                    let selection = fx.select_sound_for_footstep(phys_mat);
                    sample = if let Some(sample) = selection { sample } else { continue };
                    vol_range = (dist / 1.0).clamp(0.25, 1.5) .. 1.5;
                    speed_range = 0.75 .. 1.25;
                } else if vel_length + ang_length > 0.1 {
                    sample = if let Some(&sample) = [
                        &fx.brush1a,
                        &fx.brush1b,
                        &fx.brush1c,
                        &fx.brush1d,
                        &fx.brush1e,
                        &fx.brush1f,
                    ]
                    .choose(&mut rng) { sample.clone() } else { continue };

                    target_entity = event.collider1;
                    vol_range = (dist / 1.0).clamp(0.25, 1.0) .. 1.0;
                    speed_range = 1.0 .. 1.0;
                } else {
                    continue
                }

            } else if one_has_mat {
                let (ent, phys_mat) = if rng.random_bool(0.5) {
                    (event.collider1, phys_mat_a)
                } else {
                    (event.collider2, phys_mat_b)
                };

                target_entity = ent;
                let vol_mid = ((vel_length + ang_length) / 5.0).min(0.95);
                if vol_mid < 0.01 {
                    continue
                }

                let selection = if sliding && ang_length < vel_length /*m */ {
                    let speed_mid = ang_length / mass.0 * 100.0 / 3.0;
                    speed_range = speed_mid * 0.75 .. speed_mid * 2.0;
                    fx.select_sound_for_surface_slide(phys_mat)
                } else if vel_length > 0.1 {
                    let speed_mid = ang_length / mass.0 * 100.0 / 3.0;
                    speed_range = speed_mid * 0.75 .. speed_mid * 2.0;
                    fx.select_sound_for_surface_impact(phys_mat)
                } else {
                    continue
                };

                let Some(sound) = selection else { continue };
                sample = sound;
                vol_range = vol_mid * 0.5 .. vol_mid * 1.25;

            } else {
                continue
            };

            let vol_sel = if vol_range.is_empty() { vol_range.start } else {  rng.random_range(vol_range) };
            let vol = (impulse_log * vol_sel).clamp(0.1, 1.25);

            let speed_range = speed_range.start.max(0.25) as f64 .. speed_range.end.clamp(0.751, 3.0) as f64;
            commands.spawn((
                ChildOf(target_entity),
                Sfx,
                SamplePlayer::new(sample).with_volume(Volume::Linear(vol)),
                sample_effects![
                    SpatialBasicNode { offset: (xfrm.translation() - listener_xfrm.translation()).into(), ..default() },
                ],
                RandomPitch(speed_range),

                xfrm.clone(),
            ));

            added += 1;
            if added >= 4 {
                break
            }
        }
    }
}
