use rand::RngExt;
use rand::prelude::IndexedRandom;

use bevy::math::Affine2;
use bevy::prelude::*;

use eds_bevy_common::prelude::*;
use eds_bevy_common::physics::*;

use fedry_bevy_plugin::Scripting;
use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;

use crate::assets::*;
use crate::game::BoomMass;
use crate::game::Cube;
use crate::game::OurMidiSynth;
use crate::game::GameScript;

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
    model_assets: Res<ModelAssets>,
    scripting: Scripting::<GameScript>,
    script_assets: Res<ScriptAssets>,
) -> Result {
    commands.insert_resource(InstructionText(
        r#"
        Left Click: Fire heavy bar (hold for strength)
        Right Click: Grab and move
        "#.to_string()
    ));

    let script = scripting.new_script_from_module_id(script_assets.level_1.id(), ExecutionMode::Async)?;
    let runtime = scripting.runtime;

    let script_module = script.module();

    let cube_size = if let Some(size) = script_module.map().get(&runtime.rt.pool.for_str("block_size"))
    && let Some(size) = RtReal::try_from(&size) {
        *size as f32
    } else {
        0.75
    };

    let cube_mass = if let Ok(mass) = runtime.get_struct_value_as_number(&script_module, "block_mass") {
        mass.as_real() as f32
    } else {
        10.0f32
    };

    // Spawn cube stacks
    let collider = Collider::cuboid(1.0, 1.0, 1.0);

    let cube_gap = if let Ok(v) = runtime.get_struct_value_as_number(&script_module, "cube_gap") {
        v.as_real() as f32
    } else {
        0.02
    };

    let collision_margin = if let Ok(v) = runtime.get_struct_value_as_number(&script_module, "collision_margin") {
        v.as_real() as f32
    } else {
        cube_gap / 4.0
    };

    // let enlarge_aabb = if let Some(v) = scripting.get_struct_value_as_number(&script_module, "enlarge_aabb") {
    //     v.as_real() as f32
    // } else {
    //     0.05
    // };
    // commands.insert_resource(avian3d::collision::collider::DefaultAabbMargin(enlarge_aabb));

    let half_size = if let Ok(v) = runtime.get_struct_value_as_number(
        &script_module, "half_side_length") {
        v.as_uint() as i32
    } else {
        6
    };

    let rigid_body = if let Ok(v) = runtime.get_struct_value(
        &script_module, "static")
    && v.as_bool() {
        RigidBody::Static
    } else {
        RigidBody::Dynamic
    };

    let with_synth = if let Ok(v) = runtime.get_struct_value(
        &script_module, "with_synth") {
        v.as_bool()
    } else {
        true
    };

    let boom_mass = if let Ok(mass) = runtime.get_struct_value_as_number(
        &script_module, "boom_mass") {

        mass.as_real() as f32
    } else {
        50.0f32
    };
    commands.insert_resource(BoomMass(boom_mass));

    let axis_scale = Vec3::new(cube_size + cube_gap, cube_size + cube_gap, cube_size + cube_gap);
    let center = Vec3::new(-5.0, axis_scale.y, 5.0);

    let std_mat = materials.get(&model_assets.cube_material).unwrap().clone();

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
                        CollisionMargin(collision_margin),
                    ),

                    (
                        script.clone(),
                        SurfaceMaterial::Wood,
                    ),
                ))
                .insert_if(OurMidiSynth, || with_synth);
            }
        }
    }
    Ok(())
}
