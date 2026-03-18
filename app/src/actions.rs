

use eds_bevy_common::*;
use bevy::prelude::*;
#[cfg(feature = "input_lim")]
use leafwing_input_manager::prelude::*;
#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;

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

    input_map.insert(
        ForceLose,
        ButtonlikeChord::modified(MOD_CTRL_COMMAND, KeyCode::PageDown),
    );
    input_map.insert(
        ForceWin,
        ButtonlikeChord::modified(MOD_CTRL_COMMAND, KeyCode::PageUp),
    );

    input_map
}

#[cfg(feature = "input_bei")]
pub fn assign_extra_actions(
    mut commands: Commands,
    include: impl Bundle + Clone,
) {
    // commands.spawn((
    //     include.clone(),
    //     Action::<actions::ToggleSelect>::new(),
    //     bindings![
    //         KeyCode::AltLeft,
    //         KeyCode::AltRight,
    //         GamepadButton::LeftThumb,
    //     ],
    // ));
    commands.spawn((
        include.clone(),
        Action::<actions::ToggleGrab>::new(),
        bindings![
            MouseButton::Right,
            KeyCode::AltLeft,
            KeyCode::AltRight,
            GamepadButton::LeftTrigger2,
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<actions::CycleExtendGrab>::new(),
        Scale::splat(0.25),
        Bindings::spawn((
            Spawn((Binding::mouse_wheel(), SwizzleAxis::YYY)),
            Bidirectional::new(KeyCode::ArrowUp, KeyCode::ArrowDown),
            Bidirectional::new(GamepadButton::RightTrigger, GamepadButton::LeftTrigger),
        )),
    ));

    commands.spawn((
        include.clone(),
        Action::<actions::ChangeCamera>::new(),
        bindings![
            KeyCode::KeyV.with_mod_keys(CTRL_COMMAND),
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<actions::ForceLose>::new(),
        bindings![
            KeyCode::PageDown.with_mod_keys(CTRL_COMMAND),
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<actions::ForceWin>::new(),
        bindings![
            KeyCode::PageUp.with_mod_keys(CTRL_COMMAND),
        ],
    ));
}
