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
}


#[derive(Resource, AssetCollection)]
pub struct MapAssets {
    #[asset(path = "maps/level_0.glb#Scene0")]
    pub level_0: Handle<Scene>,
    #[asset(path = "maps/level_1.glb#Scene0")]
    pub level_1: Handle<Scene>,
}

#[derive(Resource, AssetCollection)]
pub struct SkyboxAssets {
    #[asset(path = "textures/dresden_station_night.exr")]
    pub station: Handle<Image>,
    #[asset(path = "textures/farm_field_puresky_4k.exr")]
    pub farm_field: Handle<Image>,
}

#[derive(Resource, AssetCollection)]
pub struct ModelAssets {
}

#[derive(Resource, AssetCollection)]
pub struct ScriptAssets {
    #[asset(path = "scripts/level_0.das")]
    pub level_0: Handle<ScriptModule>,
    #[asset(path = "scripts/level_1.das")]
    pub level_1: Handle<ScriptModule>,
}
