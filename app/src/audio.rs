
use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateAppExt as _;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState as _;
use bevy_asset_loader::loading_state::config::LoadingStateConfig;
// use bevy_seedling::prelude::*;

use crate::assets::FxAssets;
use crate::assets::MusicAssets;
use eds_bevy_common::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AudioCommonPlugin)

            .add_systems(Startup, initialize_audio)

            .configure_loading_state(
                LoadingStateConfig::new(ProgramState::Initializing)
                    .load_collection::<MusicAssets>()
                    .load_collection::<FxAssets>()
            )

            // .add_systems(OnEnter(LevelState::Playing),
            //     (
            //         init_background_audio,
            //     )
            // )
            // .add_systems(Update,
            //     (
            //         fade_in_background_audio
            //             .run_if(in_state(LevelState::Playing))
            //         ,
            //     )
            // )
        ;
    }
}
