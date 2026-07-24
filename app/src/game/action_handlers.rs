use std::time::Duration;
use rand::RngExt as _;
use rand::seq::IndexedRandom as _;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::ecs::system::lifetimeless::Read;
use bevy::math::Affine2;
use bevy_seedling::sample::PlaybackSettings;
use bevy_seedling::prelude::*;
use bevy::prelude::*;

#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;

use crate::assets::FxAssets;
use crate::game::*;

use eds_bevy_common::physics::*;

pub struct ActionHandlersPlugin;

impl Plugin for ActionHandlersPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FirePower>()
            .insert_resource(FirePowerWindup {
                accel: 1.1,
                max: 10.0,
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

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "game"]
pub struct FirePowerSound;

#[derive(Resource, Default, Debug, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub struct FirePower(pub f32);

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub struct FirePowerWindup {
    pub accel: f32,
    pub start: f32,
    pub max: f32,
}

impl FirePowerWindup {
    // pub(crate) fn apply_force(&self, dt: Duration, power: f32) -> f32 {
    //     let q = (dt.as_secs_f32() * 64.0).min(1.0);
    //     let mul = 1.0.lerp(self.accel, q);
    //     (power * mul).min(self.max)
    // }
    pub(crate) fn apply_force(&self, elapsed: Duration, power: f32) -> f32 {
        let duration = 1.0;

        // let now = ((elapsed.as_secs_f32() % (duration + time_margin * 2.0) - time_margin) / duration).clamp(0.0, 1.0);
        let now = (elapsed.as_secs_f32() / duration).clamp(0.0, 1.0);
        let f = EasingCurve::new(0.0f32, self.accel, EaseFunction::SmoothStepIn);
        let q = f.sample(now).unwrap();

        // let q = (power * elapsed.as_secs_f32() * 64.0).min(1.0);
        // let mul = 1.0.lerp(self.accel, q);
        // (power * mul).min(self.max)
        // (q * mul).min(self.max)

        (q + power).min(self.max)

        // elapsed.as_secs_f32()
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

#[derive(SystemParam)]
struct ActionParams<'w, 's> {
    player_q: Query<'w, 's, (Entity, Read<GlobalTransform>, Read<ColliderAabb>, Forces), With<Player>>,
    player_look_q: Query<'w, 's, Read<PlayerLook>>,

    rigid_q: Query<'w, 's, Entity, With<RigidBody>>,
    common_fx: If<Res<'w, CommonFxAssets>>,
    fx: If<Res<'w, FxAssets>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,

    mesh_params: ParamSet<'w, 's, (
        ResMut<'w, Assets<Mesh>>,
        MeshRayCast<'w, 's>,
    )>,

    world: If<Res<'w, WorldMarkerEntity>>,

    grabbed_opt: Option<Res<'w, GrabbedItem>>,

    fire_power: ResMut<'w, FirePower>,

    boom_mass: Res<'w, BoomMass>,
}

#[cfg(feature = "input_bei")]
fn on_firing_start(
    _fire: On<Start<actions::Firing>>,
    mut fire_power: ResMut<FirePower>,
    fire_power_windup: Res<FirePowerWindup>,
) {
    **fire_power = fire_power_windup.start;
}

#[cfg(feature = "input_bei")]
fn on_firing_hold(
    fire: On<Fire<actions::Firing>>,
    mut fire_power: ResMut<FirePower>,
    fire_power_windup: Res<FirePowerWindup>,
    // time: Res<Time>,
) {
    **fire_power = fire_power_windup.apply_force(Duration::from_secs_f32(fire.fired_secs), **fire_power);
}

#[cfg(feature = "input_bei")]
fn on_firing_release(
    _fire: On<Complete<actions::Firing>>,
    mut commands: Commands,
    params: Option<ActionParams>,
    mut boom_mat: Local<Handle<StandardMaterial>>,
) {
    // Fire something.
    let Some(ActionParams{
        mut player_q,
        player_look_q,
        rigid_q,
        common_fx,
        fx,
        mut materials,
        mut mesh_params,
        world,
        grabbed_opt,
        mut fire_power,
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
    let position = player_gun(&look.rotation, eyes);

    // TODO: needs to be outside character collider (i.e. measure it? configure it?).
    let launch_distance = 0.25;
    let mut pos = position + look.rotation * Vec3::NEG_Z * launch_distance;

    let ray = Ray3d::new(world_pos, look.rotation * Dir3::NEG_Z);
    let mut raycast = mesh_params.p1();
    let hits = raycast.cast_ray(ray, &MeshRayCastSettings::default()
        .always_early_exit()
        .with_visibility(RayCastVisibility::Visible),
    );
    if let Some(hit) = hits.get(0) {
        // Adjust to world (stay in local space).
        pos = position + look.rotation * Vec3::NEG_Z * (hit.1.distance.clamp(0.5, 1.5));
    }

    let xfrm = Transform::from_translation(pos).with_rotation(look.rotation);
    let power = **fire_power;

    let mat = if *boom_mat == Handle::default() {
        // let emissive = LinearRgba::new(0.5, 0.5, 0.75, 0.5);
        let mat = materials.add(StandardMaterial {
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
            // normal_map_texture: Some(fx.boom_normal.clone()),
            normal_map_texture: Some(fx.puck_normal_texture.clone()),
            .. default()
        });
        *boom_mat = mat.clone();
        mat
    } else {
        boom_mat.clone()
    };

    let rev_power = -power;
    forces.apply_linear_impulse(look.rotation * rev_power * Vec3::Z);

    do_fire(commands.reborrow(), xfrm, power, mat,
        grabbed_opt, rigid_q,
        &*common_fx, mesh_params.p0(),
        &*world, &boom_mass,
    );

    **fire_power = 0.;
}

fn do_fire(
    mut commands: Commands,

    xfrm: Transform,
    power: f32,
    boom_mat: Handle<StandardMaterial>,

    grabbed_opt: Option<Res<GrabbedItem>>,

    rigid_q: Query<Entity, With<RigidBody>>,
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

        let mut rng = rand::rng();
        let rot_y = rng.random_range(-std::f32::consts::PI .. std::f32::consts::PI);
        let xfrm = xfrm * Transform::from_rotation(Quat::from_rotation_y(rot_y));

        commands.spawn(((
            ChildOf(world.0),
            Name::new("BOOM"),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(boom_mat),
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
            LinearVelocity(vel),
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
