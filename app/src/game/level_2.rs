use bevy::prelude::*;

use eds_bevy_common::prelude::*;
use eds_bevy_common::physics::*;

use fedry_bevy_plugin::Scripting;
use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;

use crate::assets::*;
use crate::game::BoomMass;
use crate::game::Cube;
use crate::game::GameScript;

pub(crate) const ID: &str = "level2";
pub(crate) const NAME: &str = "Level 2";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_2.clone(),
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
    mut meshes: ResMut<Assets<Mesh>>,

    scripting: Scripting::<GameScript>,
    script_assets: Res<ScriptAssets>,

) -> Result {
    // Get configuration data for the initial arrangement.
    // Later, we attach this to each Cube as well.
    let script = scripting.new_script_from_module_id(script_assets.level_2.id(), ExecutionMode::Async)?;
    let runtime = scripting.runtime;

    let script_module = script.module();

    let cube_size = if let Some(size) = script_module.map().get(&runtime.rt.pool.for_str("block_size"))
    && let Some(size) = RtReal::new(&size) {
        *size as f32
    } else {
        0.75
    };

    let cube_mass = if let Some(mass) = runtime.get_struct_value_as_number(&script_module, "block_mass") {
        mass.as_real() as f32
    } else {
        10.0f32
    };

    // Spawn cube stacks
    let mat = materials.add(Color::srgb(0.2, 0.7, 0.9));
    let cube_mesh = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));

    #[allow(unused)]
    let cuboid_size = cube_size * 0.95;
    #[allow(unused)]
    let cuboid_round = (cube_size - cuboid_size) / 2.0;

    const CUBE_GAP: f32 = 0.05;
    let axis_scale = Vec3::splat(cube_size + CUBE_GAP);

    let collider = Collider::cuboid(
        cube_size as Scalar,
        cube_size as Scalar,
        cube_size as Scalar,
    );

    let size = if let Some(side_length) = runtime.get_struct_value_as_number(&script_module,
    "side_length") {

        side_length.as_uint() as i32
    } else {
        6
    };

    let rigid_body = if let Some(is_static) = runtime.get_struct_value(&script_module, "static")
    && is_static.as_bool() {
        RigidBody::Static
    } else {
        RigidBody::Dynamic
    };

    let boom_mass = if let Some(mass) = runtime.get_struct_value_as_number(&script_module, "boom_mass") {
        mass.as_real() as f32
    } else {
        50.0f32
    };
    commands.insert_resource(BoomMass(boom_mass));

    let center = if let Some(pos) = runtime.get_struct_value(&script_module, "center_pos")
        && let Some(pos) = runtime.get_vec3(&pos)  {
        pos
    } else {
        Vec3::new(12.0, axis_scale.y / 2.0, -15.0)
    };

    if let Some(data_obj) = runtime.get_struct_value(&script_module, "block_data") {
        if let Some(data) = RtStruct::new(&data_obj) {
            let data = data.map();
            let into = script.data();
            let mut into = into.map_mut();
            for (k, v) in data.iter() {
                into.insert(*k, v.clone());
            }
        } else {
            log::error!("block_data is not a Struct but {}", runtime.rt.display_obj(&data_obj));
        }
    }

    let y = 0;
    for x in 0..size
    {
        for z in 0..size
        {
            let position = Vec3::new(x as f32, y as f32, z as f32) * axis_scale + center;
            commands.spawn((
                (
                    ChildOf(world.0),
                    Name::new("CUBE"),
                    Cube,
                    Spawned,
                    CrosshairTargetable,
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(position),
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
                    LinearDamping(1.0 / 16.0),
                    AngularDamping(1.0 / 16.0),
                    Mass(cube_mass),
                    CollisionMargin(0.0),
                ),

                (script.clone(),),
            ));
        }
    }

    Ok(())
}
