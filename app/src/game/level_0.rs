use crate::assets::*;
use crate::game::BoomMass;
use crate::game::Cube;
use crate::game::GameScript;
use crate::game::load_cube_model;
use bevy::gltf::GltfMesh;
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
    });
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgramState::New), register_level)
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                    on_level_loaded.run_if(is_in_level(ID)),
            )
        ;
    }
}

fn on_level_loaded(
    mut commands: Commands,
    world: Res<WorldMarkerEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    scripting: Scripting::<GameScript>,
    script_assets: Res<ScriptAssets>,

    gltf_meshes: Res<Assets<GltfMesh>>,
    model_assets: Res<ModelAssets>,
) -> Result {

    commands.insert_resource(InstructionText(
        r#"
        Left Click: Fire heavy bar (hold for strength)
        Right Click: Grab and move
        "#.to_string()
    ));

    let script = scripting.new_script_from_module_id(script_assets.level_0.id(), ExecutionMode::Async)?;
    let runtime = scripting.runtime;

    let script_module = script.module();
    let cube_size = if let Some(size) = script_module.map().get(&runtime.rt.pool.for_str("block_size"))
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
    // let mat = materials.add(Color::srgb(0.2, 0.7, 0.9));
    // let cube_mesh = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));

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

    let (mesh, mat) = load_cube_model(&mut materials, &gltf_meshes, &model_assets)?;
    let std_mat = materials.get(&mat).ok_or(format!("failed to load material"))?.clone();

    let mut rng = rand::rng();

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
                        Cube,
                        Spawned,
                        CrosshairTargetable,
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(mat.clone()),
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

                    (script.clone(),),
                ));
            }
        }
    }

    Ok(())
}
