
use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateAppExt as _;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState as _;
use bevy_asset_loader::loading_state::config::LoadingStateConfig;
use bevy_seedling::prelude::*;
use bevy_seedling::prelude::PlaybackSettings;
use rand::RngExt;
use rand::seq::IndexedRandom as _;

use crate::assets::FxAssets;
use crate::assets::MusicAssets;
use eds_bevy_common::prelude::*;

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
        PlaybackSettings::default().remove(),
        SamplePlayer::new(
            (*[
                &music.song_1,
                &music.song_2,
                &music.song_3,
            ]
            .choose(&mut rand::rng())
            .expect("we have one"))
            .clone()
        ),
        sample_effects![
        ],
    ))
    .observe(|event: On<PlaybackCompletion>,
        mut commands: Commands,
    | {
        let entity = event.entity;
        info!("Playback elapsed on {}.", entity);
        let delay_secs = rand::rng().random_range(5 ..= 15) as f32;
        let mut delayed = commands.delayed();
        info!("Next selection begins in {delay_secs:.0} seconds");
        delayed.secs(delay_secs).run_system_cached(init_background_audio);
    })
    ;
}
