use bevy::camera::visibility::RenderLayers;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use bevy_seedling::pool::SamplerPool;
use bevy_seedling::prelude::MainBus;
use bevy_seedling::prelude::Volume;
use strum::VariantArray;

use crate::ExitRequest;
use eds_bevy_common::*;
use crate::game::Difficulty;
use crate::game::LevelDifficulty;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MenuCommonPlugin)
            .add_systems(OnEnter(OverlayState::MainMenu), on_enter_main_menu)
            .add_systems(OnEnter(OverlayState::EscapeMenu), on_enter_escape_menu)
            .add_systems(OnExit(OverlayState::EscapeMenu), on_exit_escape_menu)
            .add_systems(OnEnter(OverlayState::GameMenu), on_enter_game_menu)
            .add_systems(OnEnter(OverlayState::OptionsMenu), on_enter_options_menu)
            .add_systems(OnEnter(OverlayState::AudioMenu), on_enter_audio_menu)
            .add_systems(OnEnter(OverlayState::VideoMenu), on_enter_video_menu)
            .add_systems(OnEnter(OverlayState::ControlsMenu), on_enter_controls_menu);
    }
}

#[derive(Debug)]
pub(crate) enum SimpleMenuActions {
    // PlayGame,
    GameMenu,
    OptionsMenu,
    AudioMenu,
    VideoMenu,
    ControlsMenu,
    Quit,
    Back,
    ResumeGame,
    RestartGame,
    StopGame,
}

impl MenuItemHandler for SimpleMenuActions {
    fn handle(&mut self, world: &mut World, message: &MenuActionMessage) {
        // Fetch the paused resource into a local copy to avoid double mutable borrows.
        let mut paused_copy = world
            .get_resource::<PauseState>()
            .cloned()
            .unwrap_or_default();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);

        match message {
            MenuActionMessage::Navigate(_) => (),
            MenuActionMessage::Activate(_) | MenuActionMessage::Next(_) => match self {
                SimpleMenuActions::Back => {
                    commands.insert_resource(GoBackInMenuRequest);
                }
                // SimpleMenuActions::PlayGame => {
                //     // Do not modify current_level LevelIndex, etc. here, but in client.
                //     start_game(commands.reborrow());
                // }
                SimpleMenuActions::GameMenu => {
                    commands.insert_resource(GoIntoMenuRequest(OverlayState::GameMenu));
                }
                SimpleMenuActions::OptionsMenu => {
                    commands.insert_resource(GoIntoMenuRequest(OverlayState::OptionsMenu));
                }
                SimpleMenuActions::AudioMenu => {
                    commands.insert_resource(GoIntoMenuRequest(OverlayState::AudioMenu));
                }
                SimpleMenuActions::VideoMenu => {
                    commands.insert_resource(GoIntoMenuRequest(OverlayState::VideoMenu));
                }
                SimpleMenuActions::ControlsMenu => {
                    commands.insert_resource(GoIntoMenuRequest(OverlayState::ControlsMenu));
                }
                SimpleMenuActions::Quit => {
                    commands.insert_resource(ExitRequest);
                }
                SimpleMenuActions::ResumeGame => {
                    paused_copy.set_menu_paused(false);
                    commands.insert_resource(paused_copy);

                    commands.set_state(OverlayState::Hidden);
                }
                SimpleMenuActions::StopGame => {
                    paused_copy.set_menu_paused(false);
                    paused_copy.set_user_paused(false);
                    commands.insert_resource(paused_copy);

                    commands.set_state(ProgramState::LaunchMenu);
                    commands.set_state(GameplayState::New);
                }
                SimpleMenuActions::RestartGame => {
                    paused_copy.set_menu_paused(false);
                    // keep user_paused
                    commands.insert_resource(paused_copy);

                    commands.set_state(ProgramState::InGame);
                    commands.set_state(GameplayState::New);
                }
            },
            MenuActionMessage::Reset(_) => (),
            MenuActionMessage::Previous(_) => (),
            MenuActionMessage::Slide(..) => (),
        }
        queue.apply(world);
    }
}

fn on_enter_main_menu(
    mut commands: Commands,
    gui_assets: Res<CommonGuiAssets>,
    mut history: ResMut<MenuItemSelectionHistory>,
    // mut glyph_mats: ResMut<Assets<TitleShader>>,
    product_name: Res<ProductName>,
    level_list: Res<LevelList>,
    current_level: Option<Res<CurrentLevel>>,
) {
    // Re-initialize state (on entry and on game exit).

    // Do not clear CurrentLevel. `Play` goes there and acts as Reset...

    commands.spawn((
        DespawnOnExit(OverlayState::MainMenu),
        Text2d::new(&product_name.0),
        TextFont {
            font_size: 128.0,
            font: gui_assets.std_ui.clone(),
            ..default()
        },
        Text2dShadow {
            offset: Vec2::new(8.0, -8.0),
            color: Color::BLACK,
            ..default()
        },
        // bevy_pretty_text::prelude::Typewriter::new(30.),
        // bevy_pretty_text::prelude::Breathe {
        //     min: 0.975,
        //     max: 1.025,
        //     ..default()
        // },
        // PrettyTextMaterial(glyph_mats.add(TitleShader::default())),
        RenderLayers::layer(RENDER_LAYER_UI),
        Transform::from_xyz(0., 300.0, 0.),
    ));

    let mut builder = MenuItemBuilder::new(
        commands,
        OverlayState::MainMenu,
        ProgramState::LaunchMenu,
        gui_assets.std_ui.clone(),
        1.0,
        &history,
    );

    add_level_selector(&mut builder,
        &level_list,
        current_level.as_deref());

    // builder.add_item(
    //     if let Some(level) = current_level {
    //         format!("Reset ({})", level.label)
    //     } else {
    //         "Play".to_string()
    //     },
    //     (), SimpleMenuActions::PlayGame);

    builder.add_item("Game", (), SimpleMenuActions::GameMenu)
    .add_item("Options", (), SimpleMenuActions::OptionsMenu)
    .add_item("Quit", (), SimpleMenuActions::Quit)
    .finish(&mut history);
}

fn add_level_selector(
    builder: &mut MenuItemBuilder,
    level_list: &LevelList,
    current_level: Option<&CurrentLevel>,
) {
    fn get_level(In(entity): In<Entity>, mut enum_q: Query<&mut MenuEnum>, next_level_index: Option<Res<LevelIndex>>) {
        let index = next_level_index.map_or(0, |nli| nli.0);
        enum_q.get_mut(entity).unwrap().current = Some(index);
    }
    fn set_level(In(v): In<usize>, mut commands: Commands) {
        commands.insert_resource(LevelIndex(v));
    }
    let get_level = builder.commands().register_system(IntoSystem::into_system(get_level));
    let set_level = builder.commands().register_system(IntoSystem::into_system(set_level));

    let level_infos = level_list.0.clone();
    let current_level = current_level.cloned();
    let level_count = level_infos.len();
    let level_names = level_infos.iter().map(|info| info.label.clone()).collect::<Vec<_>>();

    builder
        .add_item(
            "Play",
            MenuEnum::new(
                get_level,
                set_level,
                move || level_count,
                move |index| {
                    if let Some(level) = &current_level && level.id == level_infos[index].id {
                        format!("{} (reset)", level_names[index])
                    } else if !level_names.is_empty() {
                        level_names[index].clone()
                    } else {
                        "???".to_string()
                    }
                }
            ),
            EnumMenuActions::SelectStartLevelEnum,
        );
}

fn on_enter_game_menu(
    gui_assets: Res<CommonGuiAssets>,
    program_state: Res<State<ProgramState>>,
    mut commands: Commands,
    mut history: ResMut<MenuItemSelectionHistory>,
    level_list: Res<LevelList>,
    current_level: Option<Res<CurrentLevel>>,
) {
    macro_rules! make_res_enum_getter_setter {
        ($getter:ident $setter:ident => $enum:ident $res:ident $field:tt) => {
            let $getter = commands.register_system(IntoSystem::into_system(
                |In(entity): In<Entity>, mut enum_q: Query<&mut MenuEnum>, res: Res<$res>| {
                    enum_q.get_mut(entity).unwrap().current = Some(
                        $enum::VARIANTS
                            .iter()
                            .position(|e| *e == res.$field)
                            .unwrap(),
                    );
                },
            ));
            let $setter = commands.register_system(IntoSystem::into_system(
                |In(v): In<usize>, mut res: ResMut<$res>| {
                    res.$field = $enum::VARIANTS[v];
                },
            ));
        };
    }

    make_res_enum_getter_setter!(get_difficulty set_difficulty => Difficulty LevelDifficulty 0);

    let mut builder = MenuItemBuilder::new(
        commands,
        OverlayState::GameMenu,
        *program_state.get(),
        gui_assets.std_ui.clone(),
        1.0,
        &history,
    );

    add_level_selector(&mut builder, &level_list, current_level.as_deref());

    builder.add_item(
        "Difficulty",
        MenuEnum::new(
            get_difficulty,
            set_difficulty,
            || Difficulty::VARIANTS.len(),
            |index| Difficulty::VARIANTS[index].to_string(),
        ),
        EnumMenuActions::DifficultyEnum,
    )
    .add_item("Back", (), SimpleMenuActions::Back)
    .finish(&mut history);
}

fn on_enter_options_menu(
    gui_assets: Res<CommonGuiAssets>,
    commands: Commands,
    program_state: Res<State<ProgramState>>,
    mut history: ResMut<MenuItemSelectionHistory>,
) {
    MenuItemBuilder::new(
        commands,
        OverlayState::OptionsMenu,
        *program_state.get(),
        gui_assets.std_ui.clone(),
        1.0,
        &history,
    )
    .add_item("Audio", (), SimpleMenuActions::AudioMenu)
    .add_item("Video", (), SimpleMenuActions::VideoMenu)
    .add_item("Controls", (), SimpleMenuActions::ControlsMenu)
    .add_item("Back", (), SimpleMenuActions::Back)
    .finish(&mut history);
}

fn on_enter_escape_menu(
    gui_assets: Option<Res<CommonGuiAssets>>,
    commands: Commands,
    mut history: ResMut<MenuItemSelectionHistory>,
    current_level: Option<Res<CurrentLevel>>,
    mut paused: ResMut<PauseState>,
) {
    let Some(gui_assets) = gui_assets else {
        return
    };
    let Some(current_level) = current_level else {
        return
    };

    // The menu sets [paused()] to true on first entry
    // by setting one of the OR inputs to that method.
    paused.set_menu_paused(true);
    MenuItemBuilder::new(
        commands,
        OverlayState::EscapeMenu,
        ProgramState::InGame,
        gui_assets.std_ui.clone(),
        1.0,
        &history,
    )
    // (), SimpleMenuActions::ResumeGame)
    .add_item("Audio", (), SimpleMenuActions::AudioMenu)
    .add_item("Video", (), SimpleMenuActions::VideoMenu)
    .add_item("Controls", (), SimpleMenuActions::ControlsMenu)
    .add_item("Stop", (), SimpleMenuActions::StopGame)
    .add_item(format!("Restart ({})", current_level.label), (), SimpleMenuActions::RestartGame)
    .add_item("Resume", (), SimpleMenuActions::ResumeGame)
    .finish(&mut history);
}

fn on_exit_escape_menu(mut pause: ResMut<PauseState>) {
    // Unpause if the menu paused.
    // (Has no effect on user pause (key event) which also counts as a pause)
    pause.set_menu_paused(false);
}

#[derive(Debug, Clone)]
pub(crate) enum SliderMenuActions {
    FovSlider,
    MoveSensitivityXSlider,
    MoveSensitivityYSlider,
    MoveSensitivityZSlider,
    TurnSensitivityXSlider,
    TurnSensitivityYSlider,
    TurnSensitivityZSlider,
    // ZoomSensitivityYSlider,
}

impl MenuItemHandler for SliderMenuActions {}

#[derive(Debug, Clone)]
pub(crate) enum EnumMenuActions {
    DifficultyEnum,
    SelectStartLevelEnum,
    AntialiasingEnum,
    // MeshQualityEnum,
    ShadowQualityEnum,
    TextureQualityEnum,
    // GlassQualityEnum,
}

impl MenuItemHandler for EnumMenuActions {
    fn handle(&mut self, world: &mut World, event: &MenuActionMessage) {
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);
        if let MenuActionMessage::Activate(_) = event
        && let EnumMenuActions::SelectStartLevelEnum = self
        {
            start_game(commands.reborrow());
        }
        queue.apply(world);
    }
}

#[derive(Debug, Clone)]
pub(crate) enum VolumeMenuActions {
    MainVolumeSlider,
    MusicVolumeSlider,
    EffectsVolumeSlider,
    UiVolumeSlider,
    // AmbientVolumeSlider,
}

impl MenuItemHandler for VolumeMenuActions {}

fn on_enter_audio_menu(
    gui_assets: Res<CommonGuiAssets>,
    mut commands: Commands,
    program_state: Res<State<ProgramState>>,
    mut history: ResMut<MenuItemSelectionHistory>,
    // mut master_vol_q: Single<&mut UserVolume, With<MainBus>>,
    // mut music_vol_q: Single<&mut UserVolume, With<SamplerPool<Music>>>,
    // mut fx_vol_q: Single<&mut UserVolume, With<SamplerPool<Sfx>>>,
    // mut ui_vol_q: Single<&mut UserVolume, With<SamplerPool<UiSfx>>>,
) {
    macro_rules! make_volume_getter_setter_mute {
        ($getter:ident $setter:ident $get_mute:ident $set_mute:ident => $bus_or_pool:path) => {
            let $getter = commands.register_system(IntoSystem::into_system(
                |In(entity): In<Entity>,
                 mut slider_q: Query<&mut MenuSlider>,
                 vol_q: Single<&mut UserVolume, With<$bus_or_pool>>| {
                    slider_q.get_mut(entity).unwrap().current = Some(vol_q.volume.linear());
                },
            ));
            let $setter = commands.register_system(IntoSystem::into_system(
                |In(v): In<f32>, mut vol_q: Single<&mut UserVolume, With<$bus_or_pool>>| {
                    vol_q.volume = Volume::Linear(v);
                },
            ));
            let $get_mute = commands.register_system(IntoSystem::into_system(
                |In(entity): In<Entity>,
                 mut toggle_q: Query<&mut MenuToggle>,
                 vol_q: Single<&mut UserVolume, With<$bus_or_pool>>| {
                    toggle_q.get_mut(entity).unwrap().current = Some(!vol_q.muted);
                },
            ));
            let $set_mute = commands.register_system(IntoSystem::into_system(
                |In(v): In<bool>, mut vol_q: Single<&mut UserVolume, With<$bus_or_pool>>| {
                    vol_q.muted = !v;
                },
            ));
        };
    }

    make_volume_getter_setter_mute!(get_master set_master get_master_muted set_master_muted => MainBus);
    make_volume_getter_setter_mute!(get_music set_music  get_music_muted set_music_muted => SamplerPool<Music>);
    make_volume_getter_setter_mute!(get_effects set_effects  get_effects_muted set_effects_muted  => SamplerPool<Sfx>);
    make_volume_getter_setter_mute!(get_ui set_ui  get_ui_muted set_ui_muted  => SamplerPool<UiSfx>);

    let make_audio_slider = |getter, setter, defval| -> MenuSlider {
        MenuSlider::new(
            getter,
            setter,
            move || defval,
            |v| (v * 100.0).round(),
            |v| v / 100.0,
            0.0..=100.0,
            5.0,
        )
    };

    MenuItemBuilder::new(
        commands,
        OverlayState::AudioMenu,
        *program_state.get(),
        gui_assets.std_ui.clone(),
        1.0,
        &history,
    )
    .add_item(
        "Master Volume",
        (
            make_audio_slider(get_master, set_master, Some(0.7)),
            MenuToggle::new(get_master_muted, set_master_muted),
        ),
        VolumeMenuActions::MainVolumeSlider,
    )
    .add_item(
        "Music Volume",
        (
            make_audio_slider(get_music, set_music, Some(0.5)),
            MenuToggle::new(get_music_muted, set_music_muted),
        ),
        VolumeMenuActions::MusicVolumeSlider,
    )
    .add_item(
        "Effects Volume",
        (
            make_audio_slider(get_effects, set_effects, Some(0.7)),
            MenuToggle::new(get_effects_muted, set_effects_muted),
        ),
        VolumeMenuActions::EffectsVolumeSlider,
    )
    .add_item(
        "UI Volume",
        (
            make_audio_slider(get_ui, set_ui, Some(1.0)),
            MenuToggle::new(get_ui_muted, set_ui_muted),
        ),
        VolumeMenuActions::UiVolumeSlider,
    )
    .add_item("Back", (), SimpleMenuActions::Back)
    .finish(&mut history);
}

enum ControlMenuToggleActions {
    TurnInvertX,
    TurnInvertY,
    // ZoomInvertY,
}

impl MenuItemHandler for ControlMenuToggleActions {}

fn on_enter_controls_menu(
    gui_assets: Res<CommonGuiAssets>,
    program_state: Res<State<ProgramState>>,
    mut commands: Commands,
    mut history: ResMut<MenuItemSelectionHistory>,
) {
    // Scales are edited logarithmically.
    fn sens_to_ui(v: f32) -> f32 {
        if v > 0.0 { v.log2() } else { 0.0 }
    }
    fn sens_from_ui(v: f32) -> f32 {
        v.exp2()
    }

    macro_rules! make_getter_setter {
        ($getter:ident $setter:ident => $field1:ident $field2:tt) => {
            let $getter = commands.register_system(IntoSystem::into_system(
                |In(entity): In<Entity>,
                 res: Res<PlayerControllerSettings>,
                 mut slider_q: Query<&mut MenuSlider>| {
                    slider_q.get_mut(entity).unwrap().current = Some(res.$field1.$field2);
                },
            ));
            let $setter = commands.register_system(IntoSystem::into_system(
                |In(v): In<f32>, mut res: ResMut<PlayerControllerSettings>| {
                    res.$field1.$field2 = v;
                },
            ));
        };
    }

    macro_rules! make_toggle {
        ($getter:ident $setter:ident => $field:ident) => {
            let $getter = commands.register_system(IntoSystem::into_system(
                |In(entity): In<Entity>,
                 mut toggle_q: Query<&mut MenuToggle>,
                 res: Res<PlayerControllerSettings>| {
                     let current = res.$field;
                    toggle_q.get_mut(entity).unwrap().current = Some(current);
                },
            ));
            let $setter = commands.register_system(IntoSystem::into_system(
                |In(v): In<bool>, mut res: ResMut<PlayerControllerSettings>| {
                    res.$field = v;
                },
            ));
        };
    }

    make_getter_setter!(get_move_x set_move_x => move_scale x);
    make_getter_setter!(get_move_y set_move_y => move_scale y);
    make_getter_setter!(get_move_z set_move_z => move_scale z);
    make_getter_setter!(get_turn_x set_turn_x => turn_scale x);
    make_getter_setter!(get_turn_y set_turn_y => turn_scale y);
    make_getter_setter!(get_turn_z set_turn_z => turn_scale z);
    // make_getter_setter!(get_zoom_y set_zoom_y => zoom_scale y);

    make_toggle!(get_invert_turn_x set_invert_turn_x => invert_turn_x);
    make_toggle!(get_invert_turn_y set_invert_turn_y => invert_turn_y);
    // make_toggle!(get_invert_zoom_y set_invert_zoom_y => invert_zoom_y);

    MenuItemBuilder::new(
        commands,
        OverlayState::ControlsMenu,
        *program_state.get(),
        gui_assets.std_ui.clone(),
        0.75,
        &history,
    )
    .add_item(
        "Move Left/Right Power",
        MenuSlider::new(
            get_move_x,
            set_move_x,
            || Some(PlayerControllerSettings::default().move_scale.x),
            sens_to_ui,
            sens_from_ui,
            -5.0f32..=5.0f32,
            0.1,
        ),
        SliderMenuActions::MoveSensitivityXSlider,
    )
    .add_item(
        "Move Up/Down Power",
        MenuSlider::new(
            get_move_y,
            set_move_y,
            || Some(PlayerControllerSettings::default().move_scale.y),
            sens_to_ui,
            sens_from_ui,
            -5.0f32..=5.0f32,
            0.1,
        ),
        SliderMenuActions::MoveSensitivityYSlider,
    )
    .add_item(
        "Move Forward/Back Power",
        MenuSlider::new(
            get_move_z,
            set_move_z,
            || Some(PlayerControllerSettings::default().move_scale.z),
            sens_to_ui,
            sens_from_ui,
            -5.0f32..=5.0f32,
            0.1,
        ),
        SliderMenuActions::MoveSensitivityZSlider,
    )
    .add_item("Invert Turn X", (
        MenuToggle::new(get_invert_turn_x, set_invert_turn_x),
    ), ControlMenuToggleActions::TurnInvertX)
    .add_item("Invert Turn Y", (
        MenuToggle::new(get_invert_turn_y, set_invert_turn_y),
    ), ControlMenuToggleActions::TurnInvertY)
    .add_item(
        "Turn X Power",
        MenuSlider::new(
            get_turn_x,
            set_turn_x,
            || Some(PlayerControllerSettings::default().turn_scale.x),
            sens_to_ui,
            sens_from_ui,
            -5.0f32..=5.0f32,
            0.1,
        ),
        SliderMenuActions::TurnSensitivityXSlider,
    )
    .add_item(
        "Turn Y Power",
        MenuSlider::new(
            get_turn_y,
            set_turn_y,
            || Some(PlayerControllerSettings::default().turn_scale.y),
            sens_to_ui,
            sens_from_ui,
            -5.0f32..=5.0f32,
            0.1,
        ),
        SliderMenuActions::TurnSensitivityYSlider,
    )
    .add_item(
        "Turn Z Power",
        MenuSlider::new(
            get_turn_z,
            set_turn_z,
            || Some(PlayerControllerSettings::default().turn_scale.z),
            sens_to_ui,
            sens_from_ui,
            -5.0f32..=5.0f32,
            0.1,
        ),
        SliderMenuActions::TurnSensitivityZSlider,
    )
    // .add_item(
    //     "Zoom Y Power",
    //     MenuSlider::new(
    //         get_zoom_y,
    //         set_zoom_y,
    //         || Some(PlayerControllerSettings::default().zoom_scale.y),
    //         sens_to_ui,
    //         sens_from_ui,
    //         -5.0f32..=5.0f32,
    //         0.1,
    //     ),
    //     SliderMenuActions::ZoomSensitivityYSlider,
    // )
    // .add_item("Invert Zoom Y", (
    //     MenuToggle::new(get_invert_zoom_y, set_invert_zoom_y),
    // ), ControlMenuToggleActions::ZoomInvertY)
    .add_item("Back", (), SimpleMenuActions::Back)
    .finish(&mut history);
}

fn on_enter_video_menu(
    gui_assets: Res<CommonGuiAssets>,
    program_state: Res<State<ProgramState>>,
    mut commands: Commands,
    mut history: ResMut<MenuItemSelectionHistory>,
) {
    let get_fov = commands.register_system(IntoSystem::into_system(
        |In(entity): In<Entity>, s: Res<VideoSettings>, mut slider_q: Query<&mut MenuSlider>| {
            slider_q.get_mut(entity).unwrap().current = Some(s.fov_degrees);
        },
    ));
    let set_fov = commands.register_system(IntoSystem::into_system(
        |In(v): In<f32>, mut commands: Commands, mut s: ResMut<VideoSettings>| {
            s.fov_degrees = v;
            commands.init_resource::<VideoCameraSettingsChanged>();
        },
    ));

    macro_rules! make_res_enum_getter_setter {
        ($getter:ident $setter:ident => $enum:ident $res:ident $field:tt) => {
            let $getter = commands.register_system(IntoSystem::into_system(
                |In(entity): In<Entity>, mut enum_q: Query<&mut MenuEnum>, res: Res<$res>| {
                    enum_q.get_mut(entity).unwrap().current = Some(
                        $enum::VARIANTS
                            .iter()
                            .position(|e| *e == res.$field)
                            .unwrap(),
                    );
                },
            ));
            let $setter = commands.register_system(IntoSystem::into_system(
                |In(v): In<usize>, mut res: ResMut<$res>, mut commands: Commands| {
                    res.$field = $enum::VARIANTS[v];
                    commands.init_resource::<VideoEffectSettingsChanged>();
                },
            ));
        };
    }

    make_res_enum_getter_setter!(get_shadow set_shadow => ShadowQuality VideoSettings shadow_quality);
    // make_res_enum_getter_setter!(get_glass set_glass => GlassQuality VideoSettings glass_quality);
    make_res_enum_getter_setter!(get_anti set_anti => Antialiasing VideoSettings antialiasing);
    // make_res_enum_getter_setter!(get_mesh_qual set_mesh_qual => MeshQuality VideoSettings mesh_quality);
    make_res_enum_getter_setter!(get_tex_qual set_tex_qual => TextureQuality VideoSettings texture_quality);

    MenuItemBuilder::new(
        commands,
        OverlayState::VideoMenu,
        *program_state.get(),
        gui_assets.std_ui.clone(),
        1.0,
        &history,
    )
    .add_item(
        "Field of View",
        MenuSlider::new(
            get_fov,
            set_fov,
            || Some(VideoSettings::default().fov_degrees),
            |v| v,
            |v| v.round(),
            5.0f32..=120.0f32,
            5.0,
        ),
        SliderMenuActions::FovSlider,
    )
    // .add_item(
    //     "Glass Refraction Quality",
    //     MenuEnum::new(
    //         get_glass,
    //         set_glass,
    //         || GlassQuality::VARIANTS.len(),
    //         |index| GlassQuality::VARIANTS[index].to_string(),
    //     ),
    //     EnumMenuActions::GlassQualityEnum,
    // )
    .add_item(
        "Antialiasing",
        MenuEnum::new(
            get_anti,
            set_anti,
            || Antialiasing::VARIANTS.len(),
            |index| Antialiasing::VARIANTS[index].to_string(),
        ),
        EnumMenuActions::AntialiasingEnum,
    )
    // .add_item(
    //     "Mesh Quality",
    //     MenuEnum::new(
    //         get_mesh_qual,
    //         set_mesh_qual,
    //         || MeshQuality::VARIANTS.len(),
    //         |index| MeshQuality::VARIANTS[index].to_string(),
    //     ),
    //     EnumMenuActions::MeshQualityEnum,
    // )
    .add_item(
        "Texture Quality",
        MenuEnum::new(
            get_tex_qual,
            set_tex_qual,
            || TextureQuality::VARIANTS.len(),
            |index| TextureQuality::VARIANTS[index].to_string(),
        ),
        EnumMenuActions::TextureQualityEnum,
    )
    .add_item(
        "Shadow Quality",
        MenuEnum::new(
            get_shadow,
            set_shadow,
            || ShadowQuality::VARIANTS.len(),
            |index| ShadowQuality::VARIANTS[index].to_string(),
        ),
        EnumMenuActions::ShadowQualityEnum,
    )
    .add_item("Back", (), SimpleMenuActions::Back)
    .finish(&mut history);
}

fn start_game(mut commands: Commands) {
    commands.set_state(OverlayState::Loading);
    commands.set_state(ProgramState::InGame);
    commands.set_state(GameplayState::AssetsLoaded);
    // commands.insert_resource(ConnectToServer);
}
