use crate::assets::*;
use crate::game::BoomMass;
use crate::game::GameScript;
use eds_bevy_common::prelude::*;

use bevy::prelude::*;

use fedry_bevy_plugin::Scripting;
use fedry_bevy_plugin::prelude::*;

pub(crate) const ID: &str = "level5";
pub(crate) const NAME: &str = "Level 5";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_5.clone(),
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

    player_xfrm_q: Query<&Transform, With<PlayerStart>>,
) -> Result {
    commands.insert_resource(InstructionText(
        r#"
        Watch.
        "#.to_string()
    ));

    // Get configuration data.
    let script = scripting.new_script_from_module_id(script_assets.level_5.id(), ExecutionMode::Async)?;
    let runtime = scripting.runtime;

    let script_module = script.module();

    let boom_mass = if let Some(mass) = runtime.get_struct_value_as_number(&script_module, "boom_mass") {
        mass.as_real() as f32
    } else {
        5000.0f32
    };
    commands.insert_resource(BoomMass(boom_mass));

    let center = player_xfrm_q.iter().next()
        .map_or_else(|| Vec3::new(12.0, 1.0, -15.0),
        |xfrm| xfrm.translation + xfrm.rotation * Vec3::NEG_Z * 5.0);

    let mat = materials.add(Color::srgb(0.2, 0.3, 0.9));
    let cube_size = 0.1;
    let cube_mesh = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));

    let count = if let Some(v) = runtime.get_struct_value_as_number(&script_module, "count") {
        v.as_uint() as usize
    } else {
        1
    };

    for y in 0..count
    {
        let position = Vec3::new(0.0, y as f32, 0.0) * cube_size + center;

        commands.spawn((
            (
                ChildOf(world.0),
                Name::new("CONTROLLER"),
                CrosshairTargetable,
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_translation(position),
            ),

            (script.clone(),),

        ));
    }
    Ok(())
}
