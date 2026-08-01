
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
use bevy_tweening::lens::TransformScaleLens;
use rand::RngExt;

use bevy::mesh::*;
use bevy::prelude::*;

use eds_bevy_common::prelude::*;
use eds_bevy_common::physics::*;
use rand::seq::IndexedRandom;

use crate::assets::FxAssets;
use crate::game::BoomMass;

pub(crate) struct FiringPlugin;

impl Plugin for FiringPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FiringState>()
            .init_resource::<FiredItemModel>()
            .init_resource::<FiredItemStyle>()
            .insert_resource(FirePowerLimits {
                accel: 1.1,
                max: 50.0,
                start: 0.1,
            })
            .add_systems(
                FixedPreUpdate,
                projectile_follow_player
                    .run_if(not(is_paused))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame)),
            )
        ;
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
                .. default()
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
            let mesh_shape = Extrusion::new(Circle{ radius }, depth);
            let mut mesh = mesh_shape.mesh().build().rotated_by(
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
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
        (power - self.start)/ (self.max - self.start)
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
        self.strength = limits.apply_force(
            fired_secs,
            self.strength);
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

#[derive(SystemParam)]
pub(crate) struct ActionParams<'w, 's> {
    player_q: Query<'w, 's, (Entity, Read<GlobalTransform>, Read<ColliderAabb>, Forces), With<Player>>,
    player_look_q: Query<'w, 's, Read<PlayerLook>>,

    rigid_q: Query<'w, 's, Entity, With<RigidBody>>,
    common_fx: If<Res<'w, CommonFxAssets>>,
    fx: If<Res<'w, FxAssets>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,

    world: If<Res<'w, WorldMarkerEntity>>,

    grabbed_opt: Option<Res<'w, GrabbedItem>>,
    ghost_q: Query<'w, 's, Entity, With<FireGhost>>,

    boom_mass: Res<'w, BoomMass>,

    firing_state: Res<'w, FiringState>,
    fire_limits: Res<'w, FirePowerLimits>,
    fired_object: ResMut<'w, FiredItemModel>,
}

const MIN_FIRE_DISTANCE: f32 = 0.333;

/// Get the position where firing starts,
/// relative to the player's body and look rotation.
pub(crate) fn get_firing_transform(
    params: &ActionParams,

) -> Option<Transform> {
    let Ok((player, player_xfrm, aabb, _forces)) = params.player_q.single() else {
        return None
    };
    let Ok(look) = params.player_look_q.get(player) else {
        return None
    };

    let world_pos = player_xfrm.translation();
    let eyes = player_eyes(world_pos, aabb, look);

    // Apply delta as firing force builds up.
    let fire_alpha = params.fire_limits.fire_power_alpha(params.firing_state.power());
    let obj_distance = (0.75 - fire_alpha * (1.0 - MIN_FIRE_DISTANCE)).max(MIN_FIRE_DISTANCE);

    let fire_pos = player_gun(&look.rotation, aabb, eyes, obj_distance);
    Some(Transform::from_translation(fire_pos).with_rotation(look.rotation))
}

/// Create the projectile entity, but not physical yet,
/// for use in pre-firing visuals.
pub(crate) fn prepare_projectile(
    mut commands: Commands,
    mut params: ActionParams,
    meshes: ResMut<Assets<Mesh>>,
) {
    // Remove old one, if any.
    if let Ok(ent) = params.ghost_q.single() {
        commands.entity(ent).despawn();
    }

    // Fire from here.
    let Some(fire_xfrm) = get_firing_transform(&mut params) else {
        return
    };

    let mat = params.fired_object.get_material(&params.fx, params.materials.into());
    let (mesh, collider) = params.fired_object.get_mesh_and_collider(meshes);

    let mut rng = rand::rng();
    let rot_y = rng.random_range(-std::f32::consts::PI .. std::f32::consts::PI);
    let xfrm = fire_xfrm.clone().rotate_local_y(rot_y);

    commands.spawn((
        (
            ChildOf((*params.world).0),

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
        FireGhost
    ));
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
                width: 5.0,
                colour: tailwind::RED_300.with_alpha(0.666).into(),
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

pub(crate) fn projectile_follow_player(
    mut params: ActionParams,
    mut xfrm_q: Query<&mut Transform>,
    spatial: SpatialQuery,
    mut commands: Commands,
    illegal_style: Res<FiredItemStyle>,
) {
    if !params.firing_state.is_active() { return }
    let Ok(ghost) = params.ghost_q.single() else { return };

    let Some(fire_xfrm) = get_firing_transform(&params) else {
        return
    };

    let is_valid = params.test_projectile_is_free(fire_xfrm, &spatial);
    if is_valid {
        illegal_style.remove_from(commands.entity(ghost));
    } else {
        illegal_style.apply_to(commands.entity(ghost));
    }

    // Move projectile up to position.
    let fire_pos = fire_xfrm.translation;

    if let Ok(mut xfrm) = xfrm_q.get_mut(ghost) {
        let new_pos = Vec3::new(
            fire_pos.x,
            (xfrm.translation.y + fire_pos.y) / 2.0,
            fire_pos.z,
        );
        xfrm.translation = new_pos;
    }
}

fn promote_projectile(
    mut ent_commands: EntityCommands,
    world_marker: Entity,
    xfrm: Transform,
    vel: Vec3,
    mass: f32,
) {
    ent_commands.insert((
        // May already be here, just confirm.
        ChildOf(world_marker),
        xfrm,

        // Basic facts for a projectile.
        (
            Spawned,
            Projectile,
            CrosshairTargetable,
        ),

        // Make it physical.
        (
            CollisionEventsEnabled,

            CollisionLayers::default(),
            LockedAxes::default(),

            Mass(mass),
            SweptCcd::new().with_filter(CcdFilter::DEFAULT),

            LinearVelocity(vel),
            AngularVelocity(Vector::new(0., vel.length() * 0.1, 0.,)),

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
    params: ActionParams,
    xfrm_q: Query<&Transform>,
) {
    // Remove old one, if any.
    let Ok(ent) = params.ghost_q.single() else { return };

    if let Ok(xfrm) = xfrm_q.get(ent) {
        let tween = Tween::new(
            EaseFunction::BounceOut,
            Duration::from_secs_f32(0.25),
            TransformPositionScaleLens {
                start: xfrm.clone(),
                end: xfrm
                    .with_translation(xfrm.translation + Vec3::NEG_Y * 2.0)
                    .with_scale(Vec3::ZERO)
                ,
            },
        ).with_cycle_completed_event(true);

        commands.entity(ent)
            .insert(TweenAnim::new(tween))
            .observe(|event: On<CycleCompletedEvent>, mut commands: Commands| {
                commands.entity(event.anim_entity).try_despawn();
            })
        ;

    } else {
        commands.entity(ent).despawn();
    }
}

impl<'w, 's> ActionParams<'w, 's> {
    pub(crate) fn test_projectile_is_free(
        &mut self,
        fire_xfrm: Transform,
        spatial: &SpatialQuery,
    ) -> bool {
        let Ok((player, _, _, _)) = self.player_q.single() else {
            return false
        };

        // let Some(fire_xfrm) = get_firing_transform(&params) else {
        //     return Err(None)
        // };

        let ray = Ray3d::new(fire_xfrm.translation, fire_xfrm.rotation * Dir3::NEG_Z);

        let excluded = if let Some(grabbed) = &self.grabbed_opt {
            vec![
                grabbed.entity,
                player,
            ]
        } else {
            vec![
                player,
            ]
        };

        let hit = spatial.cast_shape(
            &Collider::sphere(0.5),
            ray.origin, Quat::IDENTITY,
            ray.direction,
            &ShapeCastConfig::default(),
            &SpatialQueryFilter::default().with_excluded_entities(excluded),
        );

        if let Some(hit) = hit && hit.distance < MIN_FIRE_DISTANCE / 2.0 {
            // Too close.
            false
        } else {
            // Just right.
            true
        }
    }

    /// Fire the projectile currently held.
    fn do_fire(
        &mut self,
        mut commands: Commands,
        fire_xfrm: Transform,
        vel: RVec3,
        meshes: ResMut<Assets<Mesh>>,
    ) -> bool {

        if let Some(grabbed) = &self.grabbed_opt {
            // Fire the item we are holding, if it still exists.
            if self.rigid_q.contains(grabbed.entity) {
                commands.write_message(GrabbingCommand::ReleaseItems(Some(vel)));

                commands.spawn((
                    UiSfx,
                    SamplePlayer::new(self.common_fx.release.clone()),
                    VolumeNode::from_linear(0.5),
                ));

                return true
            } else {
                commands.write_message(GrabbingCommand::CancelGrabItems);
            }
            return false
        }

        let fired_ent = if let Ok(ghost) = self.ghost_q.single() {
            // No longer a ghost.
            let mut ent_commands = commands.entity(ghost);
            ent_commands.remove::<FireGhost>();
            ghost
        } else {
            // Fire a new item if no ghost.

            // This is a fallback, in case you don't use ghosts.

            let mat = self.fired_object.get_material(&self.fx, self.materials.reborrow());
            let (mesh, collider) = self.fired_object.get_mesh_and_collider(meshes);

            commands.spawn((
                Name::new("BOOM"),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                RigidBody::Dynamic,
                collider,
                CollisionMargin(0.01),
            ))
            .id()
        };

        // FIXME
        // // Add random rotation
        // let mut rng = rand::rng();
        // let rot_y = rng.random_range(-std::f32::consts::PI .. std::f32::consts::PI);
        // let xfrm = xfrm * Transform::from_rotation(Quat::from_rotation_y(rot_y));

        // Whether it was a ghost or a new item, Convert ghost to a real physics item.
        promote_projectile(commands.entity(fired_ent),
            self.world.0.0, fire_xfrm, vel, self.boom_mass.0);


        commands.spawn((
            UiSfx,
            SamplePlayer::new(self.common_fx.release.clone()),
            VolumeNode::from_linear(0.5),
        ));

        // Add a light for fun.
        // We use NoFrustumCulling to avoid bad light clipping,
        // and a child because MeshRayCast ignores ones with this component.
        commands.spawn((
            ChildOf(fired_ent),
            PointLight {
                intensity: 3200.0,
                color: (Color::hsla(30.0, 0.5, 1.0, 1.0).to_linear() * 10.0).into(),
                ..default()
            },
            NoFrustumCulling,
        ));

        true
    }

}

pub(crate) fn fire_projectile(
    In(power): In<f32>,
    mut commands: Commands,
    mut params: If<ActionParams>,
    spatial: If<SpatialQuery>,
    meshes: ResMut<Assets<Mesh>>,
) {
    // Get our prepared ghost.
    let ghost_opt = params.ghost_q.single().ok();

    // See if we can still fire from here.
    let Some(fire_xfrm) = get_firing_transform(&*params) else {
        if let Some(ghost) = ghost_opt {
            commands.entity(ghost).try_despawn();
        }
        return
    };

    if !params.test_projectile_is_free(fire_xfrm, &*spatial) {
        commands.spawn((
            UiSfx,
            SamplePlayer::new( //[
                // common_fx.cannot1.clone(),
                params.common_fx.cannot2.clone(),
                //].choose(&mut rand::rng()).unwrap().clone()
            ),
            VolumeNode::from_linear(0.5),
        ));

        // Delete the intruder.
        if let Some(ghost) = ghost_opt {
            commands.entity(ghost).try_despawn();
        }
        return
    };

    // Make the player move back. Do this first since we also
    // fetch player velocity here.

    let player_vel = {
        let Ok((_, _, _, mut forces)) = params.player_q.single_mut() else {
            return
        };

        // Apply recoil to player.
        let rev_power = -power;
        forces.apply_linear_impulse(fire_xfrm.rotation * rev_power * Vec3::Z);

        // Fetch the player's world velocity.
        forces.linear_velocity()
    };

    // Projectile takes player's motion as well as the actual firing power.
    let projectile_vel = player_vel +
        fire_xfrm.rotation * Vec3::NEG_Z * power;

    // Ensure a projectile exists, and fire it off from this position.
    params.do_fire(commands.reborrow(), fire_xfrm, projectile_vel, meshes);

}
