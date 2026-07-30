
use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::ecs::system::lifetimeless::Read;
use bevy::math::Affine2;
use bevy_seedling::nodes::core::VolumeNode;
use bevy_seedling::sample::SamplePlayer;
use rand::RngExt;

use bevy::mesh::*;
use bevy::prelude::*;

use eds_bevy_common::prelude::*;
use eds_bevy_common::physics::*;

use crate::assets::FxAssets;
use crate::game::BoomMass;

#[derive(Resource, Default)]
pub(crate) struct FiredObject {
    material: Option<Handle<StandardMaterial>>,
    mesh_collider: Option<(Handle<Mesh>, Collider)>,
}

impl FiredObject {
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
        mut meshes: Mut<Assets<Mesh>>,
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

    mesh_params: ParamSet<'w, 's, (
        ResMut<'w, Assets<Mesh>>,
        MeshRayCast<'w, 's>,
        SpatialQuery<'w, 's>,
    )>,

    world: If<Res<'w, WorldMarkerEntity>>,

    grabbed_opt: Option<Res<'w, GrabbedItem>>,

    boom_mass: Res<'w, BoomMass>,
}

pub(crate) fn fire_projectile(
    In(power): In<f32>,
    mut commands: Commands,
    params: Option<ActionParams>,
    fired_object: ResMut<FiredObject>,
) {
    // Fire something.
    let Some(ActionParams{
        mut player_q,
        player_look_q,
        rigid_q,
        common_fx,
        fx,
        materials,
        mut mesh_params,
        world,
        grabbed_opt,
        boom_mass,
    }) = params else {
        return
    };

    // Only one player...
    let Ok((player, player_xfrm, aabb, mut forces)) = player_q.single_mut() else {
        log::error!("no single Player");
        return;
    };
    let Ok(look) = player_look_q.get(player) else {
        log::error!("no PlayerLook");
        return;
    };

    let world_pos = player_xfrm.translation();
    let eyes = player_eyes(world_pos, aabb, look);
    let fire_pos = player_gun(&look.rotation, aabb, eyes);
    let player_vel = forces.linear_velocity();

    let ray = Ray3d::new(fire_pos, look.rotation * Dir3::NEG_Z);
    let spatial = mesh_params.p2();
    let hit = spatial.cast_shape(
        &Collider::sphere(0.5),
        ray.origin, Quat::IDENTITY,
        ray.direction,
        &ShapeCastConfig::default(),
        &SpatialQueryFilter::default().with_excluded_entities([player]));

    const MIN_DISTANCE: f32 = 0.333;
    if let Some(hit) = hit && hit.distance < MIN_DISTANCE / 2.0 {
        // Can't fire this close.
        commands.spawn((
            UiSfx,
            SamplePlayer::new(common_fx.cannot.clone()),
            VolumeNode::from_linear(0.5),
        ));

        return;
    }

    // Adjust to world (stay in local space).
    let pos = fire_pos + look.rotation * Vec3::NEG_Z * MIN_DISTANCE;

    let xfrm = Transform::from_translation(pos).with_rotation(look.rotation);

    let rev_power = -power;
    forces.apply_linear_impulse(look.rotation * rev_power * Vec3::Z);

    do_fire(commands.reborrow(),
        xfrm, player_vel,
        power, materials, fired_object,
        grabbed_opt, rigid_q,
        &*fx, &*common_fx, mesh_params.p0(),
        &*world, &boom_mass,
    );
}

fn do_fire(
    mut commands: Commands,

    xfrm: Transform,
    player_vel: RVec3,
    power: f32,
    mut std_mats: ResMut<Assets<StandardMaterial>>,
    mut fired_object: ResMut<FiredObject>,
    // boom_mat: Handle<StandardMaterial>,

    grabbed_opt: Option<Res<GrabbedItem>>,

    rigid_q: Query<Entity, With<RigidBody>>,
    fx: &Res<FxAssets>,
    common_fx: &Res<CommonFxAssets>,

    mut meshes: ResMut<Assets<Mesh>>,

    world: &Res<WorldMarkerEntity>,

    boom_mass: &BoomMass,
) -> bool {
    let vel = xfrm.rotation * Vec3::NEG_Z * power;
    let mut any = false;
    if let Some(grabbed) = &grabbed_opt {
        // Fire the item we are holding, if it still exists.
        if rigid_q.contains(grabbed.entity) {
            // commands.queue(WakeBody(grabbed.entity));    // sometimes crashes
            commands.write_message(GrabbingCommand::ReleaseItems(Some(vel)));
            any = true;
        } else {
            commands.write_message(GrabbingCommand::CancelGrabItems);
        }
    } else {
        // Fire a new item.

        let mat = fired_object.get_material(&fx, std_mats.reborrow());
        let (mesh, collider) = fired_object.get_mesh_and_collider(meshes.reborrow());

        let mut rng = rand::rng();
        let rot_y = rng.random_range(-std::f32::consts::PI .. std::f32::consts::PI);
        let xfrm = xfrm * Transform::from_rotation(Quat::from_rotation_y(rot_y));

        commands.spawn(((
            ChildOf(world.0),
            Name::new("BOOM"),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            xfrm,

            // ActiveCollisionHooks::MODIFY_CONTACTS,

            // Dominance(16),
            Spawned,
            Projectile,
            CrosshairTargetable,
            SurfaceMaterial::Stone,
        ),
        (
            CollisionEventsEnabled,
            LinearVelocity(vel + player_vel),
            AngularVelocity(Vector::new(0., vel.length() * 0.1, 0.,)),
            // esp. when cylindrical, try not to wobble forever

            AngularDamping(boom_mass.0.max(0.1).ln() / 10.0),
            LinearDamping(0.05),
            SleepThreshold {
                linear: 0.125,
                angular: 0.125,
            },

        ),
        (
            Mass(boom_mass.0),
            Friction::new(0.75),
            Restitution::new(0.25),
            SweptCcd::new().with_filter(CcdFilter::DEFAULT),

            RigidBody::Dynamic,
            collider,
            CollisionMargin(0.01),
        ),

        // Add a light for fun.
        // We use NoFrustumCulling to avoid bad light clipping,
        // and a child because MeshRayCast ignores ones with this component.
        children![
            (
                PointLight {
                    intensity: 3200.0,
                    color: (Color::hsla(30.0, 0.5, 1.0, 1.0).to_linear() * 10.0).into(),
                    ..default()
                },
                NoFrustumCulling,
            )
        ]
        ));
        any = true;
    }

    if any {
        commands.spawn((
            UiSfx,
            SamplePlayer::new(common_fx.swoosh.clone()),
            VolumeNode::from_linear(0.5),
        ));
    }

    any
}
