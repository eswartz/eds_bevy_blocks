
use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateAppExt as _;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState as _;
use bevy_asset_loader::loading_state::config::LoadingStateConfig;
use bevy_seedling::prelude::SamplePlayer;
use rand::seq::IndexedRandom as _;

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

            .add_systems(OnEnter(LevelState::Playing),
                (
                    init_background_audio,
                )
            )
        ;
    }
}

pub(crate) fn init_background_audio(
    mut commands: Commands,
    world_q: Single<Entity, With<WorldMarker>>,
    music: Res<MusicAssets>,
) {
    commands.spawn((
        ChildOf(*world_q),

        Music,
        SamplePlayer::new(
            (*[&music.song_1,
            &music.song_2,
            &music.song_3,]
                .choose(&mut rand::rng())
                .expect("we have one"))
            .clone())
        .looping()
    ))
    ;
}
