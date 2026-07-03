use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use fedry_bevy_plugin::asset::ScriptModule;

#[derive(Resource, AssetCollection)]
pub struct GuiAssets {
}

#[derive(Resource, AssetCollection)]
#[allow(unused)]
pub struct MusicAssets {
}

impl MusicAssets {
}

#[derive(Resource, AssetCollection)]
#[allow(unused)]
pub struct FxAssets {
    #[asset(path = "textures/boom_texture.webp")]
    pub boom_texture: Handle<Image>,
    #[asset(path = "textures/boom_normal.webp")]
    pub boom_normal: Handle<Image>,
    #[asset(path = "textures/rocky_roughness.webp")]
    pub rocky_roughness_texture: Handle<Image>,
    #[asset(path = "textures/puck.webp")]
    pub puck_diffuse_texture: Handle<Image>,
    #[asset(path = "textures/puck_normal.webp")]
    pub puck_normal_texture: Handle<Image>,
}


#[derive(Resource, AssetCollection)]
pub struct MapAssets {
    #[asset(path = "maps/level_0.glb#Scene0")]
    pub level_0: Handle<WorldAsset>,
    #[asset(path = "maps/level_1.glb#Scene0")]
    pub level_1: Handle<WorldAsset>,
    #[asset(path = "maps/level_2.glb#Scene0")]
    pub level_2: Handle<WorldAsset>,
    #[asset(path = "maps/level_3.glb#Scene0")]
    pub level_3: Handle<WorldAsset>,
    #[asset(path = "maps/level_4.glb#Scene0")]
    pub level_4: Handle<WorldAsset>,
    #[asset(path = "maps/level_5.glb#Scene0")]
    pub level_5: Handle<WorldAsset>,
    #[asset(path = "maps/level_6.glb#Scene0")]
    pub level_6: Handle<WorldAsset>,
}

#[derive(Resource, AssetCollection)]
pub struct ModelAssets {
    #[asset(path = "models/cube.glb#Mesh0/Primitive0")]
    pub cube: Handle<Mesh>,
    #[asset(path = "models/cube.glb#Material0/std")]
    pub cube_material: Handle<StandardMaterial>,
}

#[derive(Resource, AssetCollection)]
pub struct ScriptAssets {
    #[asset(path = "scripts/level_0.das")]
    pub level_0: Handle<ScriptModule>,
    #[asset(path = "scripts/level_1.das")]
    pub level_1: Handle<ScriptModule>,
    #[asset(path = "scripts/level_2.das")]
    pub level_2: Handle<ScriptModule>,
    #[asset(path = "scripts/level_3.das")]
    pub level_3: Handle<ScriptModule>,
    #[asset(path = "scripts/level_4.das")]
    pub level_4: Handle<ScriptModule>,
    #[asset(path = "scripts/level_5.das")]
    pub level_5: Handle<ScriptModule>,
    #[asset(path = "scripts/level_6.das")]
    pub level_6: Handle<ScriptModule>,
}
