use std::time::Duration;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::color::palettes::tailwind;
use bevy::ecs::system::SystemParam;
use bevy::ecs::system::lifetimeless::Read;
use bevy::math::Affine2;
use bevy_mod_outline::OutlineVolume;
use bevy_seedling::nodes::core::VolumeNode;
use bevy_seedling::sample::SamplePlayer;
use bevy_tweening::CycleCompletedEvent;
use bevy_tweening::Tween;
use bevy_tweening::TweenAnim;
use rand::RngExt;

use bevy::mesh::*;
use bevy::prelude::*;

use eds_bevy_common::physics::*;
use eds_bevy_common::prelude::*;

use crate::assets::FxAssets;
use crate::game::BoomMass;

pub(crate) struct FiringPlugin;

impl Plugin for FiringPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<OutlinesPlugin>() {
            app.add_plugins(OutlinesPlugin);
        }
        app.init_resource::<FiringState>()
            .init_resource::<FiredItemModel>()
            .init_resource::<FiredItemStyle>()
            .insert_resource(FirePowerLimits {
                accel: 1.1,
                max: 50.0,
                start: 0.1,
            })
            .add_systems(
                FixedPreUpdate,
                update_queued_projectile
                    .run_if(not(is_paused))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame)),
            );
    }
}

#[derive(Resource, Default)]
pub(crate) struct FiredItemModel {
    material: Option<Handle<StandardMaterial>>,
    mesh_collider: Option<(Handle<Mesh>, Collider)>,
}

impl FiredItemModel {
    pub(crate) fn get_material(
        &mut self,
        fx: &FxAssets,
        mut std_mats: Mut<Assets<StandardMaterial>>,
    ) -> Handle<StandardMaterial> {
        if self.material.is_none() {
            // let emissive = LinearRgba::new(0.5, 0.5, 0.75, 0.5);
            let mat = std_mats.add(StandardMaterial {
                base_color: Color::Srgba(Srgba::new(0.75, 0.6, 0.25, 1.0) * 5.0),
                // base_color_texture: Some(fx.boom_texture.clone()),
                base_color_texture: Some(fx.puck_diffuse_texture.clone()),
                reflectance: 0.25,
                // emissive,
                // emissive_exposure_weight: 0.95,
                metallic: 1.0,
                perceptual_roughness: 0.5,
                metallic_roughness_texture: Some(fx.rocky_roughness_texture.clone()),
                ior: 1.77,
                clearcoat: 1.0,
                uv_transform: Affine2::from_scale(Vec2::new(2.0, 0.5)),
                diffuse_transmission: 0.75,
                specular_transmission: 0.75,
                alpha_mode: AlphaMode::Blend,
                normal_map_texture: Some(fx.boom_normal.clone()),
                // normal_map_texture: Some(fx.puck_normal_texture.clone()),
                ..default()
            });
            self.material.replace(mat.clone());
        }
        self.material.as_ref().unwrap().clone()
    }

    pub(crate) fn get_mesh_and_collider(
        &mut self,
        mut meshes: ResMut<Assets<Mesh>>,
    ) -> (Handle<Mesh>, Collider) {
        if self.mesh_collider.is_none() {
            // let size = Vec3::new(2.0, 0.5, 0.5);
            // let size = Vec3::new(0.5, 2.0, 0.5);
            // let size = Vec3::new(2.0, 1.0, 0.25);

            // let mesh = meshes.add(Cuboid::from_size(size));
            // let collider = Collider::cuboid(size.x as Scalar, size.y as Scalar, size.z as Scalar);

            // let mesh_shape = Extrusion::new(Triangle2d{
            //     vertices: [
            //         Vec2::new(0.0, 1.0),
            //         Vec2::new(-0.5, 0.0),
            //         Vec2::new(0.5, 0.0),
            //     ],
            // }, 1.0);
            // let mesh = meshes.add(mesh_shape.mesh());
            // let collider: Collider = Collider::trimesh_from_mesh(&mesh_shape.mesh().build()).unwrap();
            let radius = 0.5;
            let depth = 0.25;
            let mesh_shape = Extrusion::new(Circle { radius }, depth);
            let mut mesh = mesh_shape
                .mesh()
                .build()
                .rotated_by(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
            ;
            mesh.generate_tangents().unwrap();

            let mesh = meshes.add(mesh);
            let collider: Collider = Collider::cylinder(radius, depth);
            // let collider = Collider::capsule((size.z / 2.0) as Scalar, (size.y - size.x) as Scalar);

            self.mesh_collider.replace((mesh, collider));
        }

        self.mesh_collider.as_ref().unwrap().clone()
    }
}

/// Marks whatever entity is going to be fired.
#[derive(Component, Default, Debug, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
#[type_path = "game"]
pub(crate) struct FireGhost;

/// The limits of a "fire" action.
#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct FirePowerLimits {
    pub accel: f32,
    pub start: f32,
    pub max: f32,
}

impl FirePowerLimits {
    /// Get the `power` mapped to the range. Interpolates beyond limits.
    pub(crate) fn fire_power_alpha(&self, power: f32) -> f32 {
        (power - self.start) / (self.max - self.start)
    }
}

/// Current state of firing.
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct FiringState {
    strength: f32,
}
impl FiringState {
    pub(crate) fn clear(&mut self) {
        self.strength = 0.;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.strength > 0.
    }

    pub(crate) fn start(&mut self, limits: &FirePowerLimits) {
        self.strength = limits.start;
    }

    pub(crate) fn update(&mut self, fired_secs: f32, limits: &FirePowerLimits) {
        self.strength = limits.apply_force(fired_secs, self.strength);
    }

    pub(crate) fn power(&self) -> f32 {
        self.strength
    }
}

impl FirePowerLimits {
    pub(crate) fn apply_force(&self, elapsed_secs: f32, power: f32) -> f32 {
        let duration = 1.0;

        let now = (elapsed_secs / duration).clamp(0.0, 1.0);
        let f = EasingCurve::new(0.0f32, self.accel, EaseFunction::SmoothStepIn);
        let q = f.sample(now).unwrap();

        (q + power).min(self.max)
    }
}

// #[derive(Component)]
// pub(crate) struct PreparedProjectile;

#[derive(SystemParam)]
pub(crate) struct FireActionState<'w, 's> {
    player_q: Query<'w, 's, (
        Entity, Read<GlobalTransform>, Read<ColliderAabb>, Forces,
    ), With<Player>>,
    player_look_q: Query<'w, 's, Read<PlayerLook>>,

    // prepared_q: Query<'w, 's, Entity, With<PreparedProjectile>>,

    rigid_q: Query<'w, 's, Entity, With<RigidBody>>,
    common_fx: If<Res<'w, CommonFxAssets>>,
    fx: If<Res<'w, FxAssets>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,

    world: If<Res<'w, WorldMarkerEntity>>,

    grabbed_opt: Option<Res<'w, GrabbedItem>>,
    ghost_q: Query<'w, 's, Entity, With<FireGhost>>,

    boom_mass: Res<'w, BoomMass>,

    firing_state: ResMut<'w, FiringState>,
    fire_limits: Res<'w, FirePowerLimits>,
    fired_object: ResMut<'w, FiredItemModel>,
}

/// Create the projectile entity, but not physical yet,
/// for use in pre-firing visuals.
pub(crate) fn prepare_projectile(
    mut commands: Commands,
    mut fire_state: FireActionState,
    meshes: ResMut<Assets<Mesh>>,
) {
    if let Ok(ent) = fire_state.ghost_q.single() {
        commands.entity(ent).despawn();
    }
    let Some(fire_xfrm) = fire_state.get_prepared_firing_transform() else {
        return;
    };
    let mat = fire_state
        .fired_object
        .get_material(&fire_state.fx, fire_state.materials.reborrow());
    let (mesh, collider) = fire_state.fired_object.get_mesh_and_collider(meshes);
    let mut rng = rand::rng();
    let rot_y = rng.random_range(-std::f32::consts::PI..std::f32::consts::PI);
    let xfrm = fire_xfrm.clone().rotate_local_y(rot_y);
    commands.spawn((
        (
            ChildOf((*fire_state.world).0),
            Name::new("BOOM"),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            xfrm,
            Spawned,
        ),
        (
            // More reliable to be Dynamic when
            // applying initial LinearVelocity, ...
            RigidBody::Dynamic,
            // ... so lock the axes to freeze it.
            LockedAxes::ALL_LOCKED,
            // And, don't collide with the player or the world.
            CollisionLayers::NONE,
            collider,
            CollisionMargin(0.01),
        ),
        // Mark this as the to-be-fired projectile.
        FireGhost,
    ));
}

pub(crate) fn update_queued_projectile(
    mut fire_state: FireActionState,
    mut commands: Commands,
    mut xfrm_q: Query<&mut Transform>,
    spatial: SpatialQuery,
    illegal_style: Res<FiredItemStyle>,
) {
    if !fire_state.firing_state.is_active() {
        return;
    }
    let Ok(ghost) = fire_state.ghost_q.single() else {
        return;
    };
    let Some(fire_xfrm) = fire_state.get_prepared_firing_transform() else {
        return;
    };
    let is_valid = fire_state.test_projectile_is_free(fire_xfrm, &spatial);
    if is_valid {
        illegal_style.remove_from(commands.entity(ghost));
    } else {
        illegal_style.apply_to(commands.entity(ghost));
    }

    if let Ok(mut xfrm) = xfrm_q.get_mut(ghost) {
        // Move projectile up to position.
        let fire_pos = fire_xfrm.translation;
        let new_pos = (xfrm.translation + fire_pos) / 2.0;
        xfrm.translation = new_pos;
        xfrm.rotation = xfrm.rotation.slerp(fire_xfrm.rotation, 0.25);
    }
}

fn promote_projectile(
    mut ent_commands: EntityCommands,
    mass: f32,
) {
    ent_commands.insert((
        // Basic facts for a projectile.
        (Spawned, Projectile, CrosshairTargetable),
        // Make it physical.
        (
            CollisionEventsEnabled,
            CollisionLayers::default(),
            LockedAxes::default(),
            Mass(mass),
            SweptCcd::new().with_filter(CcdFilter::DEFAULT),
            AngularDamping(mass.max(0.1).ln() / 2.0),
            LinearDamping(0.05),
            SleepThreshold {
                linear: 0.125,
                angular: 0.125,
            },
        ),
        // Physics material.
        (
            Friction::new(0.75),
            Restitution::new(0.25),
            SurfaceMaterial::Stone,
        ),
    ));
}

/// Abort the projectile.
pub(crate) fn cancel_projectile(
    mut commands: Commands,
    fire_state: FireActionState,
    xfrm_q: Query<&Transform>,
) {
    // Remove old one, if any.
    let Ok(ent) = fire_state.ghost_q.single() else {
        return;
    };

    if let Ok(xfrm) = xfrm_q.get(ent) {
        let tween = Tween::new(
            EaseFunction::BounceOut,
            Duration::from_secs_f32(0.25),
            TransformPositionScaleLens {
                start: xfrm.clone(),
                end: xfrm
                    .with_translation(xfrm.translation + Vec3::NEG_Y * 2.0)
                    .with_scale(Vec3::ZERO),
            },
        )
        .with_cycle_completed_event(true);

        commands.entity(ent).insert(TweenAnim::new(tween)).observe(
            |event: On<CycleCompletedEvent>, mut commands: Commands| {
                commands.entity(event.anim_entity).try_despawn();
            },
        );
    } else {
        commands.entity(ent).despawn();
    }
}

const MIN_FIRE_DISTANCE: f32 = 0.333;

impl<'w, 's> FireActionState<'w, 's> {
    // Tell how far in meters a prepared object .
    pub(crate) fn get_firing_distance(&self, fire_power: f32) -> f32 {
        // Apply delta as firing force builds up.
        let fire_alpha = self.fire_limits.fire_power_alpha(fire_power);
        (0.75 - fire_alpha * (1.0 - MIN_FIRE_DISTANCE)).max(MIN_FIRE_DISTANCE)
    }

    /// Get the position where firing starts,
    /// world space based on the player's body and look rotation.
    pub(crate) fn get_firing_transform(&self, obj_distance: f32) -> Option<Transform> {
        let Ok((player, player_xfrm, aabb, _forces)) = self.player_q.single() else {
            return None;
        };
        let Ok(look) = self.player_look_q.get(player) else {
            return None;
        };
        // Fire where we look.
        let fire_rot = look.rotation;

        let world_pos = player_xfrm.translation();
        let eyes = player_eyes(world_pos, aabb, look);

        let body_distance = 0.5;
        let fire_pos = player_gun(&fire_rot, aabb, eyes, obj_distance + body_distance);
        Some(Transform::from_translation(fire_pos).with_rotation(fire_rot))
    }

    pub(crate) fn get_prepared_firing_transform(&self) -> Option<Transform> {
        self.get_firing_transform(self.get_firing_distance(self.firing_state.power()))
    }

    pub(crate) fn test_projectile_is_free(
        &mut self,
        fire_xfrm: Transform,
        spatial: &SpatialQuery,
    ) -> bool {
        let Ok((player, _, _, _)) = self.player_q.single() else {
            return false;
        };

        let ray = Ray3d::new(fire_xfrm.translation, fire_xfrm.rotation * Dir3::NEG_Z);

        let excluded = if let Some(grabbed) = &self.grabbed_opt {
            vec![grabbed.entity, player]
        } else {
            vec![player]
        };

        let hit = spatial.cast_shape(
            &Collider::sphere(0.5),
            ray.origin,
            Quat::IDENTITY,
            ray.direction,
            &ShapeCastConfig::default(),
            &SpatialQueryFilter::default().with_excluded_entities(excluded),
        );

        if let Some(hit) = hit
            && hit.distance < MIN_FIRE_DISTANCE / 2.0
        {
            // Too close.
            false
        } else {
            // Just right.
            true
        }
    }

    pub(crate) fn fire_projectile(
        &mut self,
        mut commands: Commands,
        spatial: &SpatialQuery,
        meshes: ResMut<Assets<Mesh>>,
        xfrm_q: Query<&Transform>,
    ) {
        let fire_xfrm_opt = self.get_prepared_firing_transform();

        let ghost_opt = self.ghost_q.single().ok();
        let power = self.firing_state.power();
        self.firing_state.clear();

        let Some(mut fire_xfrm) = fire_xfrm_opt else {
            // No player?!
            if let Some(ghost) = ghost_opt {
                commands.entity(ghost).try_despawn();
            }
            return;
        };

        if let Some(ghost) = &ghost_opt {
            // See if we can still fire from here.
            if !self.test_projectile_is_free(fire_xfrm, &spatial) {
                commands.spawn((
                    UiSfx,
                    SamplePlayer::new(
                        //[
                        // common_fx.cannot1.clone(),
                        self.common_fx.cannot2.clone(),
                        //].choose(&mut rand::rng()).unwrap().clone()
                    ),
                    VolumeNode::from_linear(0.5),
                ));

                // Delete the intruder.
                commands.entity(*ghost).try_despawn();
                return;
            }
        } else if let Some(grabbed) = &self.grabbed_opt {
            let fire_rot = fire_xfrm.rotation;
            fire_xfrm = xfrm_q.get(grabbed.entity).map_or(fire_xfrm, |f| *f);
            fire_xfrm.rotation = fire_rot;
        } else {
            // New item.
        }

        // Make the player move back. Do this first since we also
        // fetch player velocity here to share the borrow.
        let player_vel = {
            let Ok((_, _, _, mut forces)) = self.player_q.single_mut() else {
                return;
            };

            // Apply recoil to player.
            let rev_power = -power;
            forces.apply_linear_impulse(fire_xfrm.rotation * rev_power * Vec3::Z);

            // Fetch the player's world velocity.
            forces.linear_velocity()
        };

        // Projectile takes player's motion as well as the actual firing power.
        let projectile_vel = player_vel + fire_xfrm.rotation * Vec3::NEG_Z * power;

        // Ensure a projectile exists, and fire it off from this position.
        let mut is_new_or_ghost = false;
        let fired_ent = if let Some(grabbed) = &self.grabbed_opt {
            // Fire the item we are holding, if it still exists.
            if self.rigid_q.contains(grabbed.entity) {
                commands.write_message(GrabbingCommand::ReleaseItems(Some(projectile_vel)));

                commands.spawn((
                    UiSfx,
                    SamplePlayer::new(self.common_fx.release.clone()),
                    VolumeNode::from_linear(0.5),
                ));
                grabbed.entity
            } else {
                commands.write_message(GrabbingCommand::CancelGrabItems);
                // We lost it!
                return
            }
        } else if let Some(ghost) = ghost_opt {
            // No longer a ghost.
            let mut ent_commands = commands.entity(ghost);
            ent_commands.remove::<FireGhost>();
            is_new_or_ghost = true;
            ghost
        } else {
            // Fire a new item, then!.

            // This is a fallback, in case you don't use ghosts.

            let mat = self
                .fired_object
                .get_material(&self.fx, self.materials.reborrow());
            let (mesh, collider) = self.fired_object.get_mesh_and_collider(meshes);

            is_new_or_ghost = true;
            commands
                .spawn((
                    Name::new("BOOM"),
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    RigidBody::Dynamic,
                    collider,
                    CollisionMargin(0.01),
                    fire_xfrm,

                ))
                .id()
        };


        if is_new_or_ghost {
            // May already be here, just confirm.
            let mut ent_commands = commands.entity(fired_ent);
            ent_commands.insert((
                ChildOf(self.world.0.0),
                fire_xfrm,
            ));

            promote_projectile(
                ent_commands.reborrow(),
                self.boom_mass.0,
            );

            // Add a light for fun.
            commands.spawn((
                ChildOf(fired_ent),
                PointLight {
                    intensity: 3200.0,
                    color: (Color::hsla(30.0, 0.5, 1.0, 1.0).to_linear() * 10.0).into(),
                    ..default()
                },
                NoFrustumCulling,
            ));
        }

        let mut ent_commands = commands.entity(fired_ent);
        ent_commands.insert((
            LinearVelocity(projectile_vel),
            AngularVelocity(Vector::new(0., projectile_vel.length() * 0.1, 0.)),
        ));

        commands.spawn((
            UiSfx,
            SamplePlayer::new(self.common_fx.release.clone()),
            VolumeNode::from_linear(0.5),
        ));
    }
}

/// This resource defines the default style for highlighted items.
/// The given components are added (and removed) as needed.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct FiredItemStyle(pub OutlineStyle);

impl Default for FiredItemStyle {
    fn default() -> Self {
        Self(OutlineStyle {
            volume: OutlineVolume {
                visible: true,
                width: 16.0,
                colour: tailwind::RED_500.with_alpha(0.666).into(),
            },
            stencil: None,
            inherit: None,
        })
    }
}

impl FiredItemStyle {
    pub(crate) fn apply_to<'a>(&self, ent_commands: EntityCommands<'a>) {
        self.0.apply_to(ent_commands);
    }
    pub(crate) fn remove_from<'a>(&self, ent_commands: EntityCommands<'a>) {
        self.0.remove_from(ent_commands);
    }
}
