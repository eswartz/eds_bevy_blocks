use crate::assets::*;
use crate::game::BoomMass;
use crate::game::GameScript;
use avian3d::math::Vector;
use bevy::math::Affine2;
use eds_bevy_common::*;

use avian3d::prelude::*;
use bevy::prelude::*;
use fedry_bevy_plugin::Scripting;
use rand::RngExt;
use rand::prelude::IndexedRandom;

use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;

pub(crate) const ID: &str = "level0";
pub(crate) const NAME: &str = "Level 0";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_0.clone(),
        // scene: assets.load("maps/school_gym.glb#Scene0"),
        // scene: assets.load("maps/classroom.glb#Scene0"),

    });
}

pub struct LevelPlugin;

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Decorate(String, Timer);

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgramState::New), register_level)
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                    on_level_loaded.run_if(is_in_level(ID)),
            )
            .add_systems(
                FixedPreUpdate,
                (
                    add_colliders.run_if(is_in_level(ID)),
                )
            )
            .add_observer(|on: On<ColliderConstructorHierarchyReady>,
                mut commands: Commands,
            | {
                commands.entity(on.entity).insert(Decorate(
                    ID.to_string(),
                    Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once),
                ));
            })
        ;
    }
}

fn add_colliders(
    mut commands: Commands,
    mut decorate_q: Query<(Entity, &mut Decorate)>,
    name_q: Query<&Name>,
    mesh_q: Query<&Mesh3d>,
    child_q: Query<&Children>,
    parent_q: Query<&ChildOf>,
    time: Res<Time>,
) {
    for (entity, mut decorate) in decorate_q.iter_mut() {
        if decorate.0 == ID && decorate.1.tick(time.delta()).just_finished() {
            commands.entity(entity).remove::<Decorate>();

            let mut decorate_info = FxHashSet::default();

            for ent in child_q.iter_descendants_depth_first(entity) {
                if let Ok(name) = name_q.get(ent)
                && (name.starts_with("SchoolChair") || name.starts_with("SchoolDesk")|| name.starts_with("PlasticChair"))
                && name.contains("_")
                && let Ok(parent) = parent_q.get(ent)
                && mesh_q.get(ent).is_ok()
                && mesh_q.get(parent.0).is_err()
                {
                    decorate_info.insert((parent.0, ent, name.to_string()));
                }
            }

            for (parent, ent, name) in decorate_info {
                let dens = ColliderDensity(10.0);
                let mut parent_ent_commands = commands.entity(parent);
                if name.starts_with("SchoolChair") {
                    parent_ent_commands.insert((
                        Mass(10.0),
                        CenterOfMass(Vector::new(0., -0.2, -0.75)),
                    ));
                }
                else if name.starts_with("PlasticChair") {
                    parent_ent_commands.insert((
                        Mass(20.0),
                        CenterOfMass(Vector::new(0., -0.5, 0.0)),
                    ));
                }
                else if name.starts_with("SchoolDesk") {
                    parent_ent_commands.insert((
                        Mass(20.0),
                        CenterOfMass(Vector::new(0., -0.5, 0.0)),
                    ));
                }
                else {
                    continue
                }
                parent_ent_commands.insert((
                    RigidBody::Dynamic,
                    CrosshairTargetable,
                    dens,
                    Friction::new(0.9),
                ));
            }
        }
    }
}

fn on_level_loaded(
    mut commands: Commands,
    world: Res<WorldMarkerEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    scripting: Scripting::<GameScript>,
    assets: Res<AssetServer>,
    script_assets: Res<ScriptAssets>,
    model_assets: Res<ModelAssets>,
    ctx_p: ConvertContextParam,
) -> Result {

    let ctx = ctx_p.as_ctx();

    commands.insert_resource(InstructionText(
        r#"
        Left Click: Fire heavy bar (hold for strength)
        Right Click: Grab and move
        "#.to_string()
    ));

    let script = scripting.new_script_from_module_id(script_assets.level_0.id(), ExecutionMode::Async)?;
    let runtime = scripting.runtime;

    let script_module = script.module();
    let cube_size = if let Some(size) = runtime.get_struct_value(&script_module, "block_size")
    && let Some(size) = RtReal::new(&size) {
        *size as f32
    } else {
        0.75
    };

    let cube_mass = if let Some(mass) = runtime.get_struct_value(&script_module, "block_mass")
    && let Some(mass) = RtNumber::new(&mass) {
        mass.as_real() as f32
    } else {
        10.0f32
    };

    // Spawn cube stacks
    #[allow(unused)]
    let cuboid_size = cube_size * 0.95;
    #[allow(unused)]
    let cuboid_round = (cube_size - cuboid_size) / 2.0;

    const CUBE_GAP: f32 = 0.05;
    let axis_scale = Vec3::splat(cube_size + CUBE_GAP);

    let collider = Collider::cuboid(1.0, 1.0, 1.0);

    let half_size = if let Some(half_side_length) = runtime.get_struct_value(&script_module,
    "half_side_length")
    && let Some(half_side_length) = RtNumber::new(&half_side_length) {
        half_side_length.as_sint() as i32
    } else {
        6
    };

    let rigid_body = if let Some(is_static) = runtime.get_struct_value(&script_module, "static")
    && is_static.as_bool() {
        RigidBody::Static
    } else {
        RigidBody::Dynamic
    };

    let boom_mass = if let Some(mass) = runtime.get_struct_value(&script_module, "boom_mass")
        && let Some(mass) = RtNumber::new(&mass) {
        mass.as_real() as f32
    } else {
        50.0f32
    };
    commands.insert_resource(BoomMass(boom_mass));

    let mut rng = rand::rng();

    let std_mat = materials.get(&model_assets.cube_material).unwrap().clone();

    if let Some(scene) = runtime.get_struct_value(&script_module, "scene")
    && let Some(scene_path) = RtString::new(&scene, &runtime.rt.pool) {
        let scene_offs = runtime.get_struct_value(&script_module, "scene_offs")
            .and_then(|offs| convert_obj_to_value::<Vec3>(&ctx, &offs).ok())
            .unwrap_or_default()
        ;
        let scene_rot = runtime.get_struct_value(&script_module, "scene_rot")
            .and_then(|offs| convert_obj_to_value::<Vec3>(&ctx, &offs).ok())
            .unwrap_or_default()
        ;
        commands.spawn((
            ChildOf(world.0),
            WorldAssetRoot(assets.load(scene_path.str().to_string())),
            Transform::from_translation(scene_offs)
                .with_rotation(
                    Quat::from_euler(EulerRot::XYZ,
                    scene_rot.x.to_radians(),
                    scene_rot.y.to_radians(),
                    scene_rot.z.to_radians()
                ))
            ,
        ));
    }

    let center = Vec3::new(-5.0, axis_scale.y / 2.0, 5.0);
    for x in -half_size..half_size {
        for y in 0..half_size * 2 {
            for z in -half_size..half_size {
                let position =
                    Vec3::new(x as f32, y as f32, z as f32) * axis_scale + center;

                let scale = Vec2::splat(rng.random_range(0.25 .. 1.5));
                let ang = rng.random_range(-0.1 .. 0.1) +
                        *[0.0, std::f32::consts::FRAC_PI_2, std::f32::consts::PI, std::f32::consts::FRAC_PI_2 * 3.0]
                            .choose(&mut rng).unwrap();
                let offs = Vec2::new(rng.random_range(0.0 .. 1.0), rng.random_range(0.0 .. 1.0));
                let mat = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.7, 0.9),
                    base_color_texture: None,
                    uv_transform: Affine2::from_scale_angle_translation(scale, ang, offs),
                    ..std_mat.clone()
                });

                commands.spawn((
                    (
                        ChildOf(world.0),
                        Name::new("CUBE"),
                        Spawned,
                        CrosshairTargetable,
                        Mesh3d(model_assets.cube.clone()),
                        MeshMaterial3d(mat),
                        Transform::from_translation(position).with_scale(Vec3::splat(cube_size)),
                    ),
                    (
                        rigid_body.clone(),
                        collider.clone(),
                        Restitution::new(0.0), //.with_combine_rule(CoefficientCombine::Min),
                        Friction::new(0.9),
                        SleepThreshold {
                            linear: 0.125,
                            angular: 0.125,
                        },
                        LinearDamping(0.625),
                        AngularDamping(0.625),
                        Mass(cube_mass),
                        CollisionMargin(CUBE_GAP / 4.0),
                    ),

                    (
                        script.clone(),
                        SurfaceMaterial::Wood,
                    ),
                ));
            }
        }
    }

    Ok(())
}
