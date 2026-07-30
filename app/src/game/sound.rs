use std::collections::VecDeque;
use std::{any::Any, num::NonZeroUsize, sync::Mutex};

use eds_bevy_common::prelude::*;
use eds_bevy_common::physics::*;

use bevy_seedling::{firewheel::Volume, prelude::*, sample::{AudioSample, SamplePlayer}};
use bevy::{math::FloatOrd, prelude::*};
use rand::seq::SliceRandom;
use rustc_hash::FxHashMap;

use lru::LruCache;
use rand::{RngExt as _, seq::IndexedRandom as _};

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(RetimedSamples::new(256).with_save_files(false))
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
struct QuantizedFloat(FloatOrd);

impl From<QuantizedFloat> for f32 {
    fn from(value: QuantizedFloat) -> Self {
        value.0.0
    }
}

impl QuantizedFloat {
    #[allow(unused)]
    pub(crate) fn rounded_to_pow2(v: f32) -> Option<Self> {
        if v <= 0.0 { return None };
        let v_l2 = v.log2();
        let res = v_l2.round().exp2();
        Some(Self(FloatOrd(res)))
    }

    #[allow(unused)]
    pub(crate) fn rounded_to_multiple(v: f32, mult: f32) -> Option<Self> {
        if v <= 0.0 { return None };
        let ret = (v / mult).ceil() * mult;
        Some(Self(FloatOrd(ret)))
    }

    pub(crate) fn as_f32(&self) -> f32 {
        (*self).into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RetimedSampleKey {
    pub(crate) scale_factor: FloatOrd,
    pub(crate) orig: Handle<AudioSample>,
}
impl RetimedSampleKey {
    fn new(source: Handle<AudioSample>, scale_factor: FloatOrd) -> Self {
        Self { scale_factor, orig: source }
    }
}

#[derive(Resource, Reflect, Clone)]
#[reflect(Resource, Default, Clone)]
pub(crate) struct RetimedSamples {
    #[reflect(ignore)]
    cache: LruCache<RetimedSampleKey, Handle<AudioSample>>,
    pub(crate) save_files: bool,
}

impl Default for RetimedSamples {
    fn default() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(1).unwrap()),
            save_files: false,
        }
    }
}

impl RetimedSamples {
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(cap.max(1)).unwrap()),
            save_files: false,
        }
    }

    pub(crate) fn with_save_files(self, save_files: bool) -> Self {
        Self {
            save_files,
            .. self
        }
    }

    /// Return a version of the [AudioSample] scaled in length (but not pitch)
    /// by [`scale_factor`].
    pub(crate) fn fetch_retimed(&mut self, mut assets: Mut<Assets<AudioSample>>, source: Handle<AudioSample>, scale_factor: f32) -> Option<Handle<AudioSample>> {
        let key = RetimedSampleKey::new(source, FloatOrd(scale_factor));
        let ret: Handle<AudioSample>;
        if let Some(target) = self.cache.get(&key) {
            ret = (*target).clone();
        } else {
            debug!("retiming {} to {:.3}", key.orig.path().map_or_else(
                    || key.orig.id().to_string(),
                    |t| format!("{t}"),
                ), key.scale_factor.0);
            if scale_factor.abs_diff_eq(&1.0, 0.001) {
                // Remember this one in the cache instead of just cloning above
                // in order to keep a record of usages (and log messages) complete.
                let cloned = key.orig.clone();
                self.cache.put(key.clone(), cloned);
                return Some(key.orig)
            }
            let Some(source) = assets.get(key.orig.id()) else {
                warn!("no {}", key.orig.id());
                return None
            };
            let Some(new) = self.retime(&key.orig, source, key.scale_factor.0) else {
                return None
            };
            ret = assets.add(new);
            self.cache.put(key, ret.clone());
        }
        Some(ret)
    }

    pub(crate) fn retime(&self, src: &Handle<AudioSample>, source: &AudioSample, time_multiplier: f32) -> Option<AudioSample> {
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

        // let (n_fft, hop_length) = (2048, 512);
        let (n_fft, hop_length) = if time_multiplier >= 4.0 {
            (16384, 2048)
        } else if time_multiplier >= 3.0 {
            (8192, 1024)
        } else if time_multiplier >= 2.0 {
            (2048, 1024)
        } else if time_multiplier >= 1.0 {
            (2048, 1024)
        } else {
            (1024, 512)
        };

        let src_frames = source.len_frames() as usize;
        let mut src_buf = vec![0.0f32; src_frames];

        let src_cnt = source.fill_buffers(
            &mut [&mut src_buf.as_mut_slice()],
            0 .. src_frames * nch, 0);

        // Accumulator of signal strength.
        let src_sum_sqr: f32 = src_buf.iter().map(|s| *s * s).sum::<f32>();

        // Get the RMS for use later.
        let src_rms = (src_sum_sqr / (1 + src_buf.len()) as f32).sqrt();

        let input_len = src_buf.len();
        let last_chunk = input_len % n_fft;
        if last_chunk != 0 {
            src_buf.resize(src_buf.len() + n_fft - last_chunk, 0.0f32);
        }

        const MIN_AMP: Volume = Volume::Decibels(-60.0);

        let target_frames = (time_multiplier as f32 * src_frames as f32).ceil() as usize;
        let mut target_samples = Vec::<f32>::with_capacity(target_frames);

        let mut builder = dasp_rs::proc::time_stretch(&src_buf[..], 1.0 / time_multiplier)
            .n_fft(n_fft)
            .hop_length(hop_length);
        match builder.compute() {
            Ok(mut ochunk) => target_samples.append(&mut ochunk),
            Err(e) => { error!("time stretch failed: {e}"); return None }
        }

        // Get RMS power of the signal after conversion.
        // Unfortunately it can differ a lot currently
        // resulting in very loud results.
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

        // Clean up trailing zeroes (just because).
        let stop_zeroes = target_samples.len() - target_samples.len() / 8;
        if let Some(index) = target_samples.iter().enumerate().rposition(
            |(idx, s)| idx < stop_zeroes || s.abs() > MIN_AMP.amp()
        ) {
            let _ = target_samples.drain(index..);
        }

        if self.save_files {
            use bwavfile::*;

            let src_name = src.path().map_or_else(
                || format!("{:?}", src.type_id()),
                |path| {
                    let path_str = path.to_string();
                    path_str[path_str.rfind('/').unwrap() + 1 ..].to_string()
                });

            let temp_path = std::env::temp_dir().join(format!("{src_name}-{time_multiplier:.3}.wav"));
            info!("writing {temp_path:?}");
            let mut file = std::fs::File::create(temp_path).unwrap();

            // note: only integer here
            let format = WaveFmt::new_pcm_mono(sample_rate.get() as _, 16);
            let w = WaveWriter::new(&mut file, format).unwrap();
            let mut frame_writer = w.audio_frame_writer().unwrap();
            let scaled = target_samples.iter().map(|s| (*s * 32767.0) as i16).collect::<Vec<_>>();
            frame_writer.write_frames(&scaled[..]).unwrap();
            // info!("wrote {}", scaled.len());
            let _ = frame_writer.end();
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
    lru: VecDeque<(SampleSelectorType, Handle<AudioSample>)>,
    limit: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SampleSelectorType {
    SurfaceImpact,
    SurfaceSlide,
    FootstepImpact,
    FootstepSlide,
}

use eds_bevy_common::prelude::surfaces;

impl SampleSelector {
    pub(crate) fn new(fx: &CommonFxAssets) -> Self {
        Self {
            impact_samples: surfaces::sounds_for_surface_impact(fx),
            slide_samples: surfaces::sounds_for_surface_slide(fx),
            foot_impact_samples: surfaces::sounds_for_footsteps_impact(fx),
            foot_slide_samples: surfaces::sounds_for_footsteps_slide(fx),

            lru: default(),
            limit: 4,
        }
    }

    pub(crate) fn set_repeat_limit(self, limit: usize) -> Self {
        Self {
            limit,
            .. self
        }
    }

    /// Pick a random sample of the given type,
    /// guaranteed to be
    pub(crate) fn pick_sample(
        &mut self,
        ty: SampleSelectorType,
        phys_mat: SurfaceMaterial,
    ) -> Option<Handle<AudioSample>> {
        let sample_set = match ty {
            SampleSelectorType::SurfaceImpact => &self.impact_samples,
            SampleSelectorType::SurfaceSlide => &self.slide_samples,
            SampleSelectorType::FootstepImpact => &self.foot_impact_samples,
            SampleSelectorType::FootstepSlide => &self.foot_slide_samples,
        };

        let samples = sample_set.get(&phys_mat)?;

        // Our little list of recent samples, so
        // we maintain uniqueness in sampling.
        let lru = &mut self.lru;
        let mut max_iters = 8;
        loop {
            let sample = samples.choose(&mut rand::rng()).cloned()?;
            let key = (ty, sample.clone());
            let lru_len = lru.len();
            if lru_len >= self.limit {
                // Forget old history.
                let _ = lru.drain(0 .. self.limit);
            }
            if !lru.contains(&key) || max_iters == 0 {
                lru.push_back(key);
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

    mut selector: ResMut<SampleSelector>,

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
                // They're moving slowly  other, ignore
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

                let norm_rel_vel = rel_vel.normalize_or_zero();
                let vel_rel_n = norm_rel_vel.dot(normal);
                let vel_rel_t = rel_vel - norm_rel_vel.dot(normal) * normal;
                let sliding_speed = vel_rel_t.length();
                let max_slide_speed = if one_is_player { 4.0 } else { 2.0 };
                let sliding = vel_rel_n.abs() < 0.25 && sliding_speed > max_slide_speed;

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

            // It's cheap to change volume.
            let vol_sel = if vol_range.is_empty() {
                vol_range.start
            } else {
                rng.random_range(vol_range)
            };
            let vol = (impulse_log * vol_sel).clamp(0.1, 1.25);

            let speed_range = speed_range.start.clamp(0.25, 0.75)
                .. speed_range.end.clamp(0.751, 1.25);

            // But changing speed means work (at least the first time for a given sample).
            let rate = rng.random_range(speed_range);
            // Reduce the actual retimed sample to a quantized amount since we have all the f32 range possible.
            // let Some(rate_quant) = QuantizedFloat::rounded_to_pow2(rate) else { continue };
            // The rate is reduced to a low fraction (a slow operation)
            // and the rest as a pitch shift.
            let Some(rate_quant) = QuantizedFloat::rounded_to_multiple(rate, 1.0 / 5.0) else { continue };
            let rate_fract = rate / rate_quant.as_f32();
            // let rate_fract = 0.0f32;

            // Stretch the sample in time.
            let retimed_sample = retimed_samples.fetch_retimed(
                samples.reborrow(),
                sample.clone(),
                rate_quant.as_f32(),
            )
            .unwrap_or_else(|| sample);

            commands.spawn((
                ChildOf(target_entity),
                Sfx,
                SamplePlayer::new(retimed_sample)
                    .with_volume(Volume::Linear(vol))
                ,
                bevy_seedling::prelude::PlaybackSettings::default()
                    .despawn()
                    .with_playback(true)
                    .with_play_from(PlayFrom::Seconds(rng.random_range(0.0 .. 0.05)))

                    // Apply the rest of the "time" stretch here.
                    // .with_speed(1.0 + rate_fract as f64)
                ,
                sample_effects![
                    // Prepopulate offset to avoid any timing problems
                    // with very short samples being spatialized as if at origin.
                    SpatialBasicNode { offset: (xfrm.translation() - listener_xfrm.translation()).into(), ..default() },
                ],
                xfrm.clone(),
            ));

            added += 1;
            if added >= 4 {
                break
            }
        }
    }
}
