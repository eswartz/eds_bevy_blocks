use eds_bevy_common::*;
use bevy::prelude::*;
use avian3d::math::AdjustPrecision as _;
#[cfg(feature = "input_lim")]
use leafwing_input_manager::prelude::*;
#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;

pub(crate) struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                FixedUpdate,
                (
                    check_actions,
                )
                    .run_if(not(is_paused))
                    .run_if(not(is_in_menu))
                    .run_if(is_level_active)
                    .run_if(not(debug_gui_wants_direct_input))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame))
                ,
            )
        ;
    }
}

fn check_actions(
    mut gravity_opt: Option<ResMut<avian3d::prelude::Gravity>>,
    #[cfg(feature = "input_bei")]
    grav_off: Query<&ActionEvents, (With<Action<game_actions::SetGravityOff>>, With<PlayerAction>)>,

    #[cfg(feature = "input_bei")]
    grav_tiny: Query<&ActionEvents, (With<Action<game_actions::SetGravityTiny>>, With<PlayerAction>)>,

    #[cfg(feature = "input_bei")]
    grav_half: Query<&ActionEvents, (With<Action<game_actions::SetGravityHalf>>, With<PlayerAction>)>,

    #[cfg(feature = "input_bei")]
    grav_normal: Query<&ActionEvents, (With<Action<game_actions::SetGravityNormal>>, With<PlayerAction>)>,

    mut player_mode: ResMut<PlayerMode>,
) {
    #[cfg(feature = "input_lim")]
    {
        // nothing
    }
    #[cfg(feature = "input_bei")]
    let Some(gravity) = gravity_opt.as_mut() else { return };

    let Some(events) = grav_off.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vec3::ZERO.adjust_precision();
        *player_mode = PlayerMode::Space;
    }
    let Some(events) = grav_tiny.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vec3::new(0.0, -1.0, 0.0).adjust_precision();
        *player_mode = PlayerMode::Space;
    }
    let Some(events) = grav_half.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vec3::new(0.0, -5.0, 0.0).adjust_precision();
        *player_mode = PlayerMode::Fps;
    }
    let Some(events) = grav_normal.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vec3::new(0.0, -9.8, 0.0).adjust_precision();
        *player_mode = PlayerMode::Fps;
    }
}

#[cfg(feature = "input_lim")]
pub fn extra_input_map() -> InputMap<UserAction> {
    use eds_bevy_common::UserAction::*;

    let mut input_map = InputMap::default();

    // // input_map.insert(ToggleHelp, KeyCode::F1);
    // input_map.insert(
    //     ToggleFps,
    //     ButtonlikeChord::modified(MOD_CTRL_COMMAND, KeyCode::KeyG),
    // ); // "G"raph
    // input_map.insert(
    //     ToggleSkybox,
    //     ButtonlikeChord::modified(MOD_CTRL_COMMAND, KeyCode::KeyB),
    // ); // "B"ackground

    input_map.insert(
        ChangeCamera,
        ButtonlikeChord::modified(MOD_CTRL_COMMAND, KeyCode::KeyV),
    ); // "V"iew

    // input_map.insert(
    //     SwitchNextAudioTrack,
    //     ButtonlikeChord::modified(MOD_CTRL_COMMAND, KeyCode::MediaTrackNext),
    // );
    // input_map.insert(
    //     SwitchPrevAudioTrack,
    //     ButtonlikeChord::modified(MOD_CTRL_COMMAND, KeyCode::MediaTrackPrevious),
    // );

    input_map
}

#[cfg(feature = "input_bei")]
pub mod game_actions {
    use super::*;

    #[derive(InputAction)]
    #[action_output(bool)]
    pub struct SetGravityNormal;

    #[derive(InputAction)]
    #[action_output(bool)]
    pub struct SetGravityOff;

    #[derive(InputAction)]
    #[action_output(bool)]
    pub struct SetGravityHalf;

    #[derive(InputAction)]
    #[action_output(bool)]
    pub struct SetGravityTiny;
}

#[cfg(feature = "input_bei")]
pub fn assign_extra_actions(
    mut commands: Commands,
    include: impl Bundle + Clone,
) {
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityNormal>::new(),
        bindings![
            KeyCode::Digit9.with_mod_keys(ModKeys::CONTROL),
            KeyCode::Numpad9.with_mod_keys(ModKeys::CONTROL),
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityOff>::new(),
        bindings![
            KeyCode::Digit0.with_mod_keys(ModKeys::CONTROL),
            KeyCode::Numpad0.with_mod_keys(ModKeys::CONTROL),
            ],
        ));
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityHalf>::new(),
        bindings![
            KeyCode::Digit5.with_mod_keys(ModKeys::CONTROL),
            KeyCode::Numpad5.with_mod_keys(ModKeys::CONTROL),
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityTiny>::new(),
        bindings![
            KeyCode::Digit1.with_mod_keys(ModKeys::CONTROL),
            KeyCode::Numpad1.with_mod_keys(ModKeys::CONTROL),
        ],
    ));

    commands.spawn((
        include.clone(),
        Action::<actions::ChangeCamera>::new(),
        bindings![
            KeyCode::KeyV.with_mod_keys(MOD_CTRL_COMMAND),
        ],
    ));
}
