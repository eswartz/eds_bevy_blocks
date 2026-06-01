use std::sync::atomic::Ordering;

use crate::assets::*;
use crate::game::BoomMass;
use crate::game::GameScript;
use eds_bevy_common::*;

use bevy::prelude::*;

use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;

pub(crate) const ID: &str = "level4";
pub(crate) const NAME: &str = "Level 4";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_4.clone(),
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

    scripting: Res<ScriptRuntime>,
    script_assets: Res<ScriptAssets>,
    modules: Res<Assets<ScriptModule>>,
    fuel: Res<ScriptFuel<GameScript>>,

    player_xfrm_q: Query<&Transform, With<PlayerStart>>,
) -> Result {
    commands.insert_resource(InstructionText(
        r#"
        Watch.
        "#.to_string()
    ));

    // Get configuration data.
    let script: Script<GameScript> = Script::new(
        &modules,
        &script_assets.level_4,
        &fuel.available,
        &scripting.rt,
        ExecutionMode::Sync,
    )?;

    fuel.available.fetch_add(50_000_000, Ordering::SeqCst);

    let script_module = script.module();

    let boom_mass = if let Some(mass) = scripting.get_struct_value(&script_module, "boom_mass")
        && let Some(mass) = RtNumber::new(&mass) {
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

    let count = if let Some(v) = scripting.get_struct_value(&script_module, "count")
        && let Some(v) = RtNumber::new(&v) {
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
