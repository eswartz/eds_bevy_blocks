
use std::sync::Arc;
use avian3d::math::*;
use eds_bevy_common::*;

use avian3d::prelude::*;
use bevy::prelude::*;

/// This is a hack!
///
/// By default it does nothing until [`ApplySleepHelper`] is set to [`true`].
pub struct GravitySleepPlugin;

impl Plugin for GravitySleepPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ApplySleepHelper>()

            .add_systems(
                FixedUpdate,
                    (
                        sleep_when_resting.run_if(|apply: Option<Res<ApplySleepHelper>>|
                            apply.is_some_and(|a| **a)),
                        reset_sleep_when_resting.run_if(resource_changed::<ApplySleepHelper>)
                    )
                    .before(PhysicsSystems::Prepare)
                    .run_if(not(is_user_paused))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame)),
            )
        ;
    }
}

/// Used to save off [GravityScale] for purposes of inducing sleeping.
#[derive(Component, Debug, Reflect, Clone, PartialEq)]
#[reflect(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct OrigGravityScale{
    pub orig_scale: Scalar,
    pub restore_next_time: bool,
}

#[derive(Resource, Default, Debug, Reflect, Clone, Deref, DerefMut)]
#[reflect(Resource, Default, Clone)]
#[type_path = "game"]
pub struct ApplySleepHelper(pub bool);

/// Try to combat physics fussiness by setting [GravityScale]
/// to 0. while items are touching others.
fn sleep_when_resting(
    // mut commands: Commands,
    commands: ParallelCommands,
    collisions: Collisions,
    forces_q: Query<Entity, (With<Spawned>, With<RigidBody>, Without<Grabbed>, With<GlobalTransform>)>,
    grav: Res<Gravity>,
    xfrm_aabb_q: Query<(&GlobalTransform, &ColliderAabb)>,
    grav_q: Query<(Option<&GravityScale>, Option<&OrigGravityScale>)>,
    time: Res<Time<Physics>>,
) {
    use std::sync::atomic::*;
    let rested = Arc::new(AtomicUsize::default());
    let awoken = Arc::new(AtomicUsize::default());
    forces_q.par_iter().for_each(|ent| {
        let mut any_under = false;

        let Ok((my_xfrm, _my_aabb)) = xfrm_aabb_q.get(ent) else { return };
        let Ok((my_grav_opt, my_orig_grav_opt)) = grav_q.get(ent) else { return };

        let my_desired_scale = my_orig_grav_opt.map_or(
            my_grav_opt.map_or(1.0, |g| g.0),
            |g| g.orig_scale);

        if my_desired_scale.abs() == 0. {
            // I don't want to move via gravity anyway.
            return
        }

        // Have we manipulated the scale?
        let my_current_base_scale = my_grav_opt.map_or(1.0, |g| g.0);

        let is_resting = my_current_base_scale.abs() < my_desired_scale.abs();
        let is_rested = my_current_base_scale.abs() == 0.;

        // Look for items we are on top of. If we're resting on something
        // and gravity would move us into it, wake up. Otherwise, go to sleep
        // so we don't constantly intersect/depenetrate it.
        let my_pos = my_xfrm.translation();
        for pair in collisions.graph().contact_pairs_with(ent) {

            if pair.is_touching() {
                let other = if pair.collider1 == ent { pair.collider2 } else { pair.collider1 };
                let Ok((other_xfrm, other_aabb)) = xfrm_aabb_q.get(other) else { continue };
                let other_pos = other_xfrm.translation();

                // Is other under us?
                if other_pos.y < my_pos.y {
                    // See if gravity would move "me" into "other". If so, consider to be resting on it.
                    let next_my_pos = my_pos + grav.0.adjust_precision() * my_current_base_scale * time.delta_secs();
                    if other_aabb.contains(&ColliderAabb::from_min_max(next_my_pos.adjust_precision(), next_my_pos.adjust_precision())) {
                        any_under = true;
                        if !is_rested {
                            commands.command_scope(|mut commands| {
                                commands.entity(ent).insert(OrigGravityScale{
                                    orig_scale: my_desired_scale,
                                    restore_next_time: false,
                                });
                                let mut new_scale = my_current_base_scale * 0.5;
                                if new_scale.abs() < 0.01 {
                                    new_scale = 0.;
                                }
                                commands.entity(ent).insert(GravityScale(new_scale));
                                debug!("resting {ent}");
                            });
                            rested.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    break;
                }
            }
        }

        // Nothing is under us, so re-apply gravity.
        if !any_under && is_resting {
            commands.command_scope(|mut commands| {
                if let Some(orig) = my_orig_grav_opt {
                    if !orig.restore_next_time {
                        commands.entity(ent).insert(OrigGravityScale{
                            orig_scale: orig.orig_scale,
                            restore_next_time: true,
                        });
                    } else {
                        debug!("waking {ent}");

                        commands.entity(ent).insert(GravityScale(my_desired_scale));
                        commands.entity(ent).try_remove::<(
                            OrigGravityScale,
                        )>();
                    }
                }
            });
            awoken.fetch_add(1, Ordering::SeqCst);
        }
    });

    let rested = rested.load(Ordering::SeqCst);
    let awoken = awoken.load(Ordering::SeqCst);
    if rested != 0 || awoken != 0 {
        debug!("new rested {rested}, awoken {awoken}");
    }
}

/// Clear our overrides when the ApplySleepHelper resource changes.
fn reset_sleep_when_resting(
    commands: ParallelCommands,
    grav_q: Query<(Entity, Option<&GravityScale>, Option<&OrigGravityScale>), With<RigidBody>>,
) {
    grav_q.par_iter().for_each(|(ent, my_grav_opt, my_orig_grav_opt)| {

        let my_desired_scale = my_orig_grav_opt.map_or(
            my_grav_opt.map_or(1.0, |g| g.0),
            |g| g.orig_scale);

        if my_orig_grav_opt.is_some() {
            commands.command_scope(|mut commands| {
                let mut ent_commands = commands.entity(ent);
                if my_desired_scale != 1.0 {
                    ent_commands.insert(GravityScale(my_desired_scale));
                } else {
                    ent_commands.try_remove::<(OrigGravityScale, GravityScale)>();
                }
            });
        }
    });
}
