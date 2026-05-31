use crate::assets::*;
use crate::game::BoomMass;
use crate::game::Cube;
use crate::game::OurMidiSynth;
use crate::game::GameScript;
use crate::game::load_cube_model;
use bevy::gltf::GltfMesh;
use bevy::math::Affine2;
use eds_bevy_common::*;

use avian3d::prelude::*;
use bevy::prelude::*;

use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::RtNumber;
use fedry_runtime::prelude::RtReal;
use fedry_runtime::prelude::RtSInt;
use rand::RngExt;
use rand::prelude::IndexedRandom;

pub(crate) const ID: &str = "level1";
pub(crate) const NAME: &str = "Level 1";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_1.clone(),
    });
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgramState::New), register_level)
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                    on_level_loaded.run_if(is_in_level(ID))
            )
        ;
    }
}

fn on_level_loaded(
    mut commands: Commands,
    world: Res<WorldMarkerEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltf_meshes: Res<Assets<GltfMesh>>,

    scripting: Res<ScriptRuntime>,
    model_assets: Res<ModelAssets>,
    script_assets: Res<ScriptAssets>,
    modules: Res<Assets<ScriptModule>>,
    fuel: Res<ScriptFuel<GameScript>>,
) -> Result {
    commands.insert_resource(InstructionText(
        r#"
        Left Click: Fire heavy bar (hold for strength)
        Right Click: Grab and move
        "#.to_string()
    ));

    let script: Script<GameScript> = Script::new(
        &modules,
        &script_assets.level_1,
        &fuel.available,
        &scripting.rt,
        ExecutionMode::Async,
    )?;

    let script_module = script.module();

    let cube_size = if let Some(size) = script_module.map().get(&scripting.rt.pool.for_str("block_size"))
    && let Some(size) = RtReal::new(&size) {
        *size as f32
    } else {
        0.75
    };

    let cube_mass = if let Some(mass) = scripting.get_struct_value(&script_module, "block_mass")
    && let Some(mass) = RtNumber::new(&mass) {
        mass.as_real() as f32
    } else {
        10.0f32
    };

    // Spawn cube stacks
    let (mesh, mat) = load_cube_model(&mut materials, &gltf_meshes, &model_assets)?;
    let std_mat = materials.get(&mat).ok_or(format!("failed to load material"))?.clone();

    let collider = Collider::cuboid(1.0, 1.0, 1.0);

    let cube_gap = if let Some(v) = scripting.get_struct_value(&script_module, "cube_gap")
    && let Some(v) = RtNumber::new(&v) {
        v.as_real() as f32
    } else {
        0.02
    };

    let collision_margin = if let Some(v) = scripting.get_struct_value(&script_module, "collision_margin")
    && let Some(v) = RtNumber::new(&v) {
        v.as_real() as f32
    } else {
        cube_gap / 4.0
    };

    // let enlarge_aabb = if let Some(v) = scripting.get_struct_value(&script_module, "enlarge_aabb")
    // && let Some(v) = RtNumber::new(&v) {
    //     v.as_real() as f32
    // } else {
    //     0.05
    // };
    // commands.insert_resource(avian3d::collision::collider::DefaultAabbMargin(enlarge_aabb));

    let half_size = if let Some(v) = scripting.get_struct_value(
        &script_module, "half_side_length")
    && let Some(v) = RtSInt::new(&v) {
        *v as i32
    } else {
        6
    };

    let rigid_body = if let Some(v) = scripting.get_struct_value(
        &script_module, "static")
    && v.as_bool() {
        RigidBody::Static
    } else {
        RigidBody::Dynamic
    };

    let with_synth = if let Some(v) = scripting.get_struct_value(
        &script_module, "with_synth") {
        v.as_bool()
    } else {
        true
    };

    let boom_mass = if let Some(mass) = scripting.get_struct_value(
        &script_module, "boom_mass")
    && let Some(mass) = RtNumber::new(&mass) {
        mass.as_real() as f32
    } else {
        50.0f32
    };
    commands.insert_resource(BoomMass(boom_mass));

    let axis_scale = Vec3::new(cube_size + cube_gap, cube_size + cube_gap, cube_size + cube_gap);
    let center = Vec3::new(-5.0, axis_scale.y / 2.0, 5.0);

    let mut rng = rand::rng();
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
                        CollisionMargin(collision_margin),
                    ),

                    (script.clone(),),
                ))
                .insert_if(OurMidiSynth, || with_synth);
            }
        }
    }
    Ok(())
}
