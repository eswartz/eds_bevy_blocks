use crate::assets::*;
use crate::game::BoomMass;
use crate::game::GameScript;
use anyhow::anyhow;
use bevy::math::Affine2;
use bevy::world_serialization::WorldInstanceReady;
use eds_bevy_common::physics::*;
use eds_bevy_common::prelude::*;

use bevy::prelude::*;
use fedry_bevy_plugin::Scripting;
use rand::RngExt;
use rand::prelude::IndexedRandom;

use fedry_bevy_plugin::prelude::*;

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

#[derive(Resource)]
struct MakeDynamic(Vec<Entity>);

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgramState::New), register_level)
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                on_level_loaded.run_if(is_in_level(ID)),
            )
            .add_systems(
                FixedLast,
                (
                    // add_colliders.run_if(is_in_level(ID)),
                    make_dynamic.run_if(is_in_level(ID)),
                ),
            )
            .add_observer(
                |on: On<WorldInstanceReady>,
                 mut commands: Commands,
                 mesh_q: Query<&Mesh3d>,
                 child_q: Query<&Children>,
                 parent_q: Query<&ChildOf>,
                 name_q: Query<&Name>| {
                    let mut ents = vec![];
                    for ent in child_q.iter_descendants_depth_first(on.entity) {
                        if let Ok(name) = name_q.get(ent)
                            && (name.starts_with("SchoolChair")
                                || name.starts_with("SchoolDesk")
                                || name.starts_with("PlasticChair")
                                || name.starts_with("LibraryChair"))
                            && name.contains("_")
                            && let Ok(parent) = parent_q.get(ent)
                            && mesh_q.get(ent).is_ok()
                            && mesh_q.get(parent.0).is_err()
                        {
                            commands.entity(ent).insert(
                                ColliderConstructor::ConvexDecompositionFromMeshWithConfig(
                                    VhacdParameters {
                                        resolution: 128,                 // Higher = more detail (but slower)
                                        concavity: 0.01, // Lower = more parts but better fit
                                        max_convex_hulls: 32, // Maximum number of convex parts
                                        plane_downsampling: 4, // Precision of plane search
                                        convex_hull_downsampling: 4, // Precision of convex hull generation
                                        alpha: 0.05, // Bias toward symmetrical splits
                                        beta: 0.05,  // Bias toward revolution axis splits
                                        convex_hull_approximation: true, // Approximate for speed
                                        fill_mode: FillMode::FloodFill {
                                            detect_cavities: false,
                                        },
                                        ..default()
                                    },
                                ),
                            );
                            ents.push(ent);
                        }
                    }
                    if !ents.is_empty() {
                        commands.insert_resource(MakeDynamic(ents));
                    }
                },
            );
    }
}

fn make_dynamic(
    mut commands: Commands,
    mut make: If<ResMut<MakeDynamic>>,
    cc_q: Query<(&ComputedCenterOfMass, &ComputedAngularInertia)>,
    mut prev: Local<Option<Entity>>,
) {
    if let Some(&ent) = (*make).0.first() {
        if cc_q.contains(ent) {
            let _ = (*make).0.remove(0);
            commands.entity(ent).insert(RigidBody::Dynamic);
            commands.entity(ent).remove::<GravityScale>();
            *prev = Some(ent);
        }
    } else {
        commands.remove_resource::<MakeDynamic>();
    }
    if let Some(prev) = prev.take() {
        commands.entity(prev).remove::<Sleeping>();
    }
}

fn on_level_loaded(
    mut commands: Commands,
    world: Res<WorldMarkerEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    scripting: Scripting<GameScript>,
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
        "#
        .to_string(),
    ));

    let script =
        scripting.new_script_from_module_id(script_assets.level_0.id(), ExecutionMode::Async)?;
    let runtime = scripting.runtime;

    let script_module = script.module();
    let cube_size =
        if let Ok(size) = runtime.get_struct_value_as_number(&script_module, "block_size") {
            size.as_real() as f32
        } else {
            0.75
        };

    let cube_mass =
        if let Ok(mass) = runtime.get_struct_value_as_number(&script_module, "block_mass") {
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

    let half_size = if let Ok(half_side_length) =
        runtime.get_struct_value_as_number(&script_module, "half_side_length")
    {
        half_side_length.as_int() as i32
    } else {
        6
    };

    let rigid_body = if let Ok(is_static) = runtime.get_struct_value(&script_module, "static")
        && is_static.as_bool()
    {
        RigidBody::Static
    } else {
        RigidBody::Dynamic
    };

    let boom_mass =
        if let Ok(mass) = runtime.get_struct_value_as_number(&script_module, "boom_mass") {
            mass.as_real() as f32
        } else {
            50.0f32
        };
    commands.insert_resource(BoomMass(boom_mass));

    let mut rng = rand::rng();

    let std_mat = materials.get(&model_assets.cube_material).unwrap().clone();

    if let Ok(scene_path) = runtime.get_struct_value_as_string(&script_module, "scene") {
        let scene_offs = runtime
            .get_struct_value(&script_module, "scene_offs")
            .map_err(|e| anyhow!(e))
            .and_then(|offs| convert_obj_to_value::<Vec3>(&ctx, &offs))
            .unwrap_or_default();
        let scene_rot = runtime
            .get_struct_value(&script_module, "scene_rot")
            .map_err(|e| anyhow!(e))
            .and_then(|offs| convert_obj_to_value::<Vec3>(&ctx, &offs))
            .unwrap_or_default();
        commands.spawn((
            ChildOf(world.0),
            WorldAssetRoot(assets.load(scene_path.to_string())),
            Transform::from_translation(scene_offs).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                scene_rot.x.to_radians(),
                scene_rot.y.to_radians(),
                scene_rot.z.to_radians(),
            )),
        ));
    }

    let center = Vec3::new(-5.0, axis_scale.y / 2.0, 5.0);
    for x in -half_size..half_size {
        for y in 0..half_size * 2 {
            for z in -half_size..half_size {
                let position = Vec3::new(x as f32, y as f32, z as f32) * axis_scale + center;

                let scale = Vec2::splat(rng.random_range(0.25..1.5));
                let ang = rng.random_range(-0.1..0.1)
                    + *[
                        0.0,
                        std::f32::consts::FRAC_PI_2,
                        std::f32::consts::PI,
                        std::f32::consts::FRAC_PI_2 * 3.0,
                    ]
                    .choose(&mut rng)
                    .unwrap();
                let offs = Vec2::new(rng.random_range(0.0..1.0), rng.random_range(0.0..1.0));
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
                    (script.clone(), SurfaceMaterial::Wood),
                ));
            }
        }
    }

    Ok(())
}
