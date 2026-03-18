
use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateAppExt as _;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState as _;
use bevy_asset_loader::loading_state::config::LoadingStateConfig;
use bevy_seedling::prelude::*;

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
            .add_systems(Update,
                (
                    spawn_menu_fx,
                    handle_menu_actions,
                )
            )
        ;
    }
}

fn initialize_audio(master: Single<Entity, With<MainBus>>, mut commands: Commands) {
    commands.entity(*master).insert(UserVolume {
        volume: Volume::Linear(0.5),
        muted: false,
    });

    const DEFAULT_POOL_VOLUME: Volume = Volume::Linear(1.0);

    // For each new pool, we can provide non-default initial values for the volume.
    commands.spawn((
        Name::new("Music"),
        SamplerPool(Music),
        UserVolume {
            volume: DEFAULT_POOL_VOLUME,
            muted: false,
        },
        PoolSize(2 ..= 4),

        MusicBus,

        // So we can apply fading.
        sample_effects![
            VolumeNode::default(),
        ],

    ))
    ;
    commands.spawn((
        Name::new("SFX"),
        SamplerPool(Sfx),
        sample_effects![(
            SpatialBasicNode {
                panning_threshold: 0.9,
                ..default()
            },
        )],
        UserVolume {
            volume: DEFAULT_POOL_VOLUME,
            muted: false,
        },
        PoolSize(8 ..= 32),
    ));
    commands.spawn((
        Name::new("UI"),
        SamplerPool(UiSfx),
        UserVolume {
            volume: DEFAULT_POOL_VOLUME,
            muted: false,
        },
        PoolSize(2 ..= 8),
    ));
}


#[derive(Component, Reflect)]
#[reflect(Component)]
struct BackgroundAudio;

// /// Add background music, which resets when the game starts or stops.
// #[allow(unused)]
// pub(crate) fn init_background_audio(
//     mut commands: Commands,
//     music: Res<MusicAssets>,
//     // track_q: Query<&MusicTrack, With<LevelRoot>>,
//     world_q: Res<WorldMarkerEntity>,
// ) {
//     // if let Ok(track) = track_q.single() {
//         // let mut rng = rand::rng();
//         // let sample = music.get_for(track).clone();
//         let sample = music.main.clone();

//         commands.spawn((
//             ChildOf(world_q.0),
//             DespawnOnExit(GameplayState::Playing),

//             Name::new("Background Audio"),
//             BackgroundAudio,
//             Music,
//             SamplePlayer::new(sample).looping(),
//             PlaybackSettings {
//                 // play_from: PlayFrom::Seconds(rng.random_range(0.0 .. 5.0 * 60.0)),
//                 ..default()
//             },
//             sample_effects![
//                 VolumeNode::from_linear(0.)
//             ],
//         ));
//     // } else {
//     //     log::warn!("no MusicTrackSelection");
//     // }
// }

// #[allow(unused)]
// fn fade_in_background_audio(
//     mut commands: Commands,
//     bg_q: Single<&SampleEffects, Added<BackgroundAudio>>,
//     volume_nodes: Query<&VolumeNode>,
// ) {
// use std::time::Duration;
// use bevy_seedling::prelude::PlaybackSettings;
// use bevy_tweening::EaseMethod;
// use bevy_tweening::Tween;
// use bevy_tweening::TweenAnim;

//     // // TODO: file issue, can't pause or restart this...?
//     // let fade_duration = DurationSeconds(15.0);

//     // let mut events = volume_nodes.get_effect_mut(&bg_q).unwrap();
//     // volume.fade_to(Volume::UNITY_GAIN, fade_duration, &mut events);

//     let fx = *bg_q;

//     for fx_ent in fx.iter() {
//         if volume_nodes.contains(fx_ent) {
//             let tween = Tween::new(
//                 EaseMethod::EaseFunction(EaseFunction::Linear),
//                 Duration::from_secs_f32(15.0),
//                 VolumeNodeLens {
//                     start: VolumeNode::from_linear(0.),
//                     end: VolumeNode::from_linear(1.),
//                 }
//             );
//             commands.entity(fx_ent).try_insert((
//                 TweenAnim::new(tween).with_destroy_on_completed(true),
//             ));
//         }
//     }
// }

fn spawn_menu_fx(mut commands: Commands,
    fx: Option<Res<CommonFxAssets>>,
    mut reader: MessageReader<MenuActionMessage>,
) {
    if reader.is_empty() {
        return
    }
    let Some(fx) = fx else { return };

    let any = reader.read().any(is_menu_action_click_bait);

    if any {
        commands.spawn((
            UiSfx,
            SamplePlayer::new(fx.action.clone()),
        ));
    }
}

fn handle_menu_actions(mut commands: Commands,
    fx: Option<Res<CommonFxAssets>>,
    mut reader: MessageReader<MenuActionMessage>,
) {
    if reader.is_empty() {
        return
    }
    let Some(fx) = fx else { return };

    // See if a menu action happened and play a click
    let any = reader.read().any(is_menu_action_click_bait);

    if any {
        commands.spawn((
            UiSfx,
            SamplePlayer::new(fx.action.clone()),
        ));
    }
}

/// Play a click sound on menu action?
fn is_menu_action_click_bait(event: &MenuActionMessage) -> bool {
    match event {
        MenuActionMessage::Activate(_) => false,
        MenuActionMessage::Navigate(_) |
        // MenuActionMessage::Activate(_) |
        MenuActionMessage::Next(_) |
        MenuActionMessage::Reset(_) | MenuActionMessage::Previous(_) => true,
        MenuActionMessage::Slide(..) => false,
    }
}
