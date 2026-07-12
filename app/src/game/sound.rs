use std::{any::Any, num::NonZeroUsize, sync::Mutex};

use avian3d::{dynamics::rigid_body::{AngularVelocity, mass_properties::components::Mass}, prelude::{Collisions, LinearVelocity}};
use bevy_seedling::{firewheel::Volume, prelude::*, sample::{AudioSample, SamplePlayer}};
use eds_bevy_common::*;
use bevy::{math::FloatOrd, prelude::*};
use rustc_hash::FxHashMap;

use lru::LruCache;
use rand::{RngExt as _, seq::IndexedRandom as _};
use timestretch::{EdmPreset, QualityMode, StreamProcessor, StretchParams};

pub(crate) struct SoundPlugin;

const SAVE_STRETCHED_FILES: bool = false;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(RetimedSamples::new(128))
            .add_systems(OnEnter(ProgramState::LaunchMenu), init_samples)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PowerOfTwo(FloatOrd);

impl PowerOfTwo {
    pub(crate) fn rounded_to_pow2(v: f32) -> Option<Self> {
        if v <= 0.0 { return None };
        let v_l2 = v.log2();
        Some(Self(FloatOrd(v_l2.round().exp2())))
    }

    pub(crate) fn as_f32(&self) -> f32 {
        self.0.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RetimedSampleKey {
    pub(crate) scale_factor: PowerOfTwo,
    pub(crate) orig: Handle<AudioSample>,
}
impl RetimedSampleKey {
    fn new(source: Handle<AudioSample>, scale_factor: PowerOfTwo) -> Self {
        Self { scale_factor, orig: source }
    }
}

#[derive(Resource)]
pub(crate) struct RetimedSamples {
    cache: LruCache<RetimedSampleKey, Handle<AudioSample>>,
}

impl RetimedSamples {
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(cap.max(1)).unwrap()),
        }
    }

    pub(crate) fn fetch(&mut self, mut assets: Mut<Assets<AudioSample>>, source: Handle<AudioSample>, scale_factor: PowerOfTwo) -> Option<Handle<AudioSample>> {
        if scale_factor.as_f32() == 1.0 {
            return Some(source)
        }

        let key = RetimedSampleKey::new(source, scale_factor);
        let ret: Handle<AudioSample>;
        if let Some(target) = self.cache.get(&key) {
            ret = (*target).clone();
        } else {
            info!("retiming {} to {:.3}", key.orig.id(), key.scale_factor.as_f32());
            let Some(source) = assets.get(key.orig.id()) else {
                warn!("no {}", key.orig.id());
                return None
            };
            let Some(new) = Self::retime(&key.orig, source, key.scale_factor.as_f32()) else {
                return None
            };
            ret = assets.add(new);
            self.cache.put(key, ret.clone());
        }
        Some(ret)
    }

    pub(crate) fn retime(src: &Handle<AudioSample>, source: &AudioSample, time_multiplier: f32) -> Option<AudioSample> {
        let source = &*source.get();
        let nch = source.num_channels().get();
        if nch != 1 {
            warn!("unsupported # channels {nch}");
            return None
        }
        let Some(sample_rate) = source.sample_rate() else {
            warn!("unknown sample rate");
            return None
        };

        let src_frames = source.len_frames() as usize;

        const TAIL: usize = 4096;

        let tail_frames = (time_multiplier.max(1.0) as f32 * TAIL as f32).ceil() as usize;
        let target_frames = (time_multiplier as f32 * src_frames as f32).ceil() as usize;
        let mut target_samples = Vec::<f32>::with_capacity(target_frames + tail_frames);

        let params = StretchParams::new(time_multiplier as _)
            .with_preset(EdmPreset::Halftime)
            .with_sample_rate(sample_rate.get() as _)
            .with_quality_mode(QualityMode::MaxQuality)
            .with_channels(1);

        let mut stretcher = StreamProcessor::new(params);

        // Process the file in chunks.
        const BUF_SIZE: usize = 4096;
        let mut src_buf = [0.0f32; BUF_SIZE];

        let mut start_frame = 0;

        let mut src_sum_sqr: f32 = 0.0;

        while start_frame < src_frames {
            let cnt = source.fill_buffers(&mut [&mut src_buf], 0 .. BUF_SIZE, start_frame as u64);
            if cnt == 0 {
                break
            }
            start_frame += cnt;

            src_sum_sqr += src_buf[0..cnt].iter().map(|s| *s * s).sum::<f32>();

            if let Err(e) = stretcher.process_into(&src_buf[0..cnt], &mut target_samples) {
                error!("failed to retime: {e}");
                return None
            }
        }

        let src_rms = (src_sum_sqr / (1 + start_frame) as f32).sqrt();

        if time_multiplier > 1.0 {
            // Ensure the sample is completed to avoid unwanted high-pitched tail.
            let zeroes = [0.0f32; TAIL];
            if let Err(e) = stretcher.process_into(&zeroes[..], &mut target_samples) {
                error!("failed to retime: {e}");
                return None
            }
        }

        if let Err(e) = stretcher.flush_into(&mut target_samples) {
            error!("failed to retime: {e}");
            return None
        }

        if time_multiplier > 1.0 {
            // Clip tail.
            let _ = target_samples.drain(target_samples.len() - tail_frames ..);
        }

        let target_rms = {
            let sum: f32 = target_samples.iter().map(|s| *s * s).sum();
            (sum / (1 + target_samples.len()) as f32).sqrt()
        };

        if target_rms > 0.0 && (src_rms - target_rms).abs() > 0.1 {
            let scale = src_rms / target_rms;
            for t in &mut target_samples {
                *t *= scale;
            }
        }

        if SAVE_STRETCHED_FILES {
            use bwavfile::*;

            let src_name = src.path().map_or_else(
                || format!("{:?}", src.type_id()),
                |path| {
                    let path_str = path.to_string();
                    path_str[path_str.rfind('/').unwrap() + 1 ..].to_string()
                });

            let temp_path = std::env::temp_dir().join(format!("{src_name}-{time_multiplier}.wav"));
            info!("writing {temp_path:?}");
            let mut file = std::fs::File::create(temp_path).unwrap();
            let format = WaveFmt::new_pcm_mono(sample_rate.get() as _, 32);
            let w = WaveWriter::new(&mut file, format).unwrap();
            let mut frame_writer = w.audio_frame_writer().unwrap();
            frame_writer.write_frames(&target_samples[..]).unwrap();
        }

        let resource: Vec<Vec<f32>> = vec![target_samples].into();
        let target = AudioSample::new(resource, sample_rate);
        Some(target)
    }

}


type SurfaceSampleMap = FxHashMap<SurfaceMaterial, Vec<Handle<AudioSample>>>;

#[derive(Resource)]
pub(crate) struct SampleSelector {
    pub(crate) impact_samples: SurfaceSampleMap,
    pub(crate) slide_samples: SurfaceSampleMap,
    pub(crate) foot_impact_samples: SurfaceSampleMap,
    pub(crate) foot_slide_samples: SurfaceSampleMap,
    pub(crate) lru: Mutex<Vec<(SampleSelectorType, Handle<AudioSample>)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SampleSelectorType {
    SurfaceImpact,
    SurfaceSlide,
    FootstepImpact,
    FootstepSlide,
}

use eds_bevy_common::assets::surfaces;

impl SampleSelector {
    pub(crate) fn new(fx: &CommonFxAssets) -> Self {
        Self {
            impact_samples: surfaces::sounds_for_surface_impact(fx),
            slide_samples: surfaces::sounds_for_surface_slide(fx),
            foot_impact_samples: surfaces::sounds_for_footsteps_impact(fx),
            foot_slide_samples: surfaces::sounds_for_footsteps_slide(fx),

            lru: default(),
        }
    }

    pub(crate) fn pick_sample(&self, ty: SampleSelectorType, phys_mat: SurfaceMaterial) -> Option<Handle<AudioSample>> {
        let sample_set = match ty {
            SampleSelectorType::SurfaceImpact => &self.impact_samples,
            SampleSelectorType::SurfaceSlide => &self.slide_samples,
            SampleSelectorType::FootstepImpact => &self.foot_impact_samples,
            SampleSelectorType::FootstepSlide => &self.foot_slide_samples,
        };

        let samples = sample_set.get(&phys_mat)?;

        let mut lru = self.lru.lock().ok()?;
        let mut max_iters = 8;
        loop {
            let sample = samples.choose(&mut rand::rng()).cloned()?;
            let key = (ty, sample.clone());
            let lru_len = lru.len();
            if lru_len >= samples.len() {
                let _ = lru.drain(0 .. samples.len() - 1);
            }
            if max_iters == 0 || lru.last() != Some(&key) {
                lru.push(key);
                return Some(sample)
            }
            max_iters -= 1;
        }
    }
}

fn init_samples(mut commands: Commands, fx: Res<CommonFxAssets>) {
    let selector = SampleSelector::new(&*fx);
    commands.insert_resource(selector);
}

fn spawn_noise_on_collision(
    surf_mat_q: Query<&SurfaceMaterial>,

    collisions: Collisions,
    phys_info_q: Query<(&GlobalTransform, &LinearVelocity, &AngularVelocity, &Mass)>,
    listener_q: Query<&GlobalTransform, With<SpatialListener3D>>,
    player_q: Query<&Player>,
    parent_q: Query<&ChildOf>,
    paused: Res<PhysicsPaused>,

    selector: Res<SampleSelector>,

    mut samples: ResMut<Assets<AudioSample>>,
    mut retimed_samples: ResMut<RetimedSamples>,

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
            let vol_range: core::ops::Range<f32>;
            let speed_range: core::ops::Range<f32>;
            let sample_ty: SampleSelectorType;
            let phys_mat: SurfaceMaterial;

            if one_is_player {
                // Player footsteps

                const FOOTFALL_TIMES_SAMPLE_DIST: f32 = 3.0;
                let dist = vel_length * time.delta_secs();
                *footstep_dist += dist;
                if *footstep_dist < 0.0 {
                    continue
                }

                *footstep_dist -= FOOTFALL_TIMES_SAMPLE_DIST;

                // Footsteps follow the player.
                (target_entity, phys_mat) = if player_a {
                    (event.collider1, phys_mat_b)
                } else {
                    (event.collider1, phys_mat_a)
                };

                if !sliding {
                    sample_ty = SampleSelectorType::FootstepImpact;
                    vol_range = (dist / 1.0).clamp(0.25, 1.5) .. 1.51;
                    speed_range = 0.75 .. 1.25;
                } else if vel_length + ang_length > 0.1 {
                    sample_ty = SampleSelectorType::FootstepSlide;
                    vol_range = (dist / 1.0).clamp(0.25, 1.25) .. 1.26;
                    speed_range = 0.75 .. 1.25;
                } else {
                    continue
                }

            } else if one_has_mat {
                // Object-object interaction.

                (target_entity, phys_mat) = if rng.random_bool(0.5) {
                    (event.collider1, phys_mat_a)
                } else {
                    (event.collider2, phys_mat_b)
                };

                let vol_mid = ((vel_length + ang_length) / 5.0).min(0.95);
                if vol_mid < 0.01 {
                    continue
                }

                let speed_mid = ang_length / mass.0 * 200.0 / 3.0;
                speed_range = (speed_mid * 0.75).max(0.5) .. (speed_mid * 2.0).min(2.0);

                if sliding && ang_length < vel_length /*m */ {
                    sample_ty = SampleSelectorType::SurfaceImpact;
                } else if vel_length > 0.1 {
                    sample_ty = SampleSelectorType::SurfaceSlide;
                } else {
                    continue
                };

                vol_range = vol_mid * 0.5 .. vol_mid * 1.25;
            } else {
                continue
            };

            let Some(sample) = (*selector).pick_sample(sample_ty, phys_mat) else { continue };

            let vol_sel = if vol_range.is_empty() {
                vol_range.start
            } else {
                rng.random_range(vol_range)
            };
            let vol = (impulse_log * vol_sel).clamp(0.1, 1.25);

            let speed_range = speed_range.start.clamp(0.25, 0.75)
                .. speed_range.end.clamp(0.751, 1.25);

            let rate = rng.random_range(speed_range);
            let Some(rate_pow2) = PowerOfTwo::rounded_to_pow2(rate) else { continue };
            let rate_fract = rate / rate_pow2.as_f32();

            let retimed_sample = retimed_samples.fetch(
                samples.reborrow(),
                sample.clone(),
                rate_pow2)
                .unwrap_or_else(|| sample);

            commands.spawn((
                ChildOf(target_entity),
                Sfx,
                SamplePlayer::new(retimed_sample)
                    .with_volume(Volume::Linear(vol))
                ,
                sample_effects![
                    SpatialBasicNode { offset: (xfrm.translation() - listener_xfrm.translation()).into(), ..default() },
                ],
                RandomPitch((rate_fract - 0.01) as f64 .. (rate_fract + 0.01) as f64),

                xfrm.clone(),
            ));

            added += 1;
            if added >= 4 {
                break
            }
        }
    }
}
