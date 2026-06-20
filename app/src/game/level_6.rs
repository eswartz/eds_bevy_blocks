use crate::assets::*;
use crate::game::BoomMass;
use crate::game::GameScript;
use avian3d::prelude::*;
use bevy::scene::SceneInstanceReady;
use eds_bevy_common::*;

use bevy::prelude::*;

use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;

pub(crate) const ID: &str = "level6";
pub(crate) const NAME: &str = "Level 6";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_6.clone(),
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
    assets: ResMut<AssetServer>,
) -> Result {
    commands.insert_resource(InstructionText(
        r#"
        "#.to_string()
    ));

    // Get configuration data.
    let script: Script<GameScript> = Script::new(
        &modules,
        &script_assets.level_6,
        &fuel.available,
        &scripting.rt,
        ExecutionMode::Async,
    )?;

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
        |xfrm| xfrm.translation + xfrm.rotation * Vec3::NEG_Z * 5.0 + Vec3::Y * 2.0);

    let model = if let Some(p) = scripting.get_struct_value(&script_module, "model")
        && let Some(p) = RtString::new(&p, &scripting.rt.pool) {
        Some(p.str().to_string())
    } else {
        None
    };

    let data = script.data();
    for (k, v) in script_module.map().iter() {
        data.map_mut().insert(k.clone(), v.clone());
    }

    let root = commands.spawn((
        ChildOf(world.0),
        Name::new("CONTROLLER"),
        CrosshairTargetable,

        RigidBody::Kinematic,
        Collider::cuboid(0.1, 0.1, 0.1),
        Transform::from_translation(center).with_scale(Vec3::splat(0.1)),

        (script.clone(),),

    )).id();

    if let Some(model) = model {
        commands.entity(root).insert((
            SceneRoot(assets.load::<Scene>(&model)),
        ))
        .observe(|ev: On<SceneInstanceReady>,
            child_q: Query<&Children>,
            mut xfrm_q: Query<&mut Transform, With<Mesh3d>>,
        | {
            for ent in child_q.iter_descendants(ev.entity) {
                if let Ok(mut xfrm) = xfrm_q.get_mut(ent) {
                    xfrm.scale = Vec3::splat(0.1);
                    break;
                }
            }
        });

    } else {
        let mat = materials.add(Color::srgb(0.2, 0.3, 0.9));
        let cube_size = 0.1;
        let cube_mesh = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));
        commands.entity(root).insert((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(mat.clone()),
        ));
    }


    Ok(())
}
