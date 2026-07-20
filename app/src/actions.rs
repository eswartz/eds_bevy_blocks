use eds_bevy_common::*;
use bevy::prelude::*;
#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;
use eds_bevy_common::physics::*;

pub(crate) struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
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
    grav_off: Query<&ActionEvents, (With<Action<game_actions::SetGravityOff>>, With<ActionOf<PlayerContext>>)>,

    #[cfg(feature = "input_bei")]
    grav_tiny: Query<&ActionEvents, (With<Action<game_actions::SetGravityTiny>>, With<ActionOf<PlayerContext>>)>,

    #[cfg(feature = "input_bei")]
    grav_half: Query<&ActionEvents, (With<Action<game_actions::SetGravityHalf>>, With<ActionOf<PlayerContext>>)>,

    #[cfg(feature = "input_bei")]
    grav_normal: Query<&ActionEvents, (With<Action<game_actions::SetGravityNormal>>, With<ActionOf<PlayerContext>>)>,

    mut player_mode: ResMut<PlayerMode>,
) {
    #[cfg(feature = "input_bei")]
    let Some(gravity) = gravity_opt.as_mut() else { return };

    let Some(events) = grav_off.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vector::ZERO;
        *player_mode = PlayerMode::Space;
    }
    let Some(events) = grav_tiny.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vector::new(0.0, -1.0, 0.0);
        *player_mode = PlayerMode::Space;
    }
    let Some(events) = grav_half.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vector::new(0.0, -5.0, 0.0);
        *player_mode = PlayerMode::Fps;
    }
    let Some(events) = grav_normal.iter().next() else { return };
    if events.contains(ActionEvents::START) {
        gravity.0 = Vector::new(0.0, -9.8, 0.0);
        *player_mode = PlayerMode::Fps;
    }
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
    // We need this when mod keys distinguish actions.
    let consume = ActionSettings {
        consume_input: true,
        ..default()
    };
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityNormal>::new(),
        consume.clone(),
        bindings![
            KeyCode::Digit9.with_mod_keys(MOD_CTRL_COMMAND),
            KeyCode::Numpad9.with_mod_keys(MOD_CTRL_COMMAND),
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityOff>::new(),
        consume.clone(),
        bindings![
            KeyCode::Digit0.with_mod_keys(MOD_CTRL_COMMAND),
            KeyCode::Numpad0.with_mod_keys(MOD_CTRL_COMMAND),
            ],
        ));
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityHalf>::new(),
        consume.clone(),
        bindings![
            KeyCode::Digit5.with_mod_keys(MOD_CTRL_COMMAND),
            KeyCode::Numpad5.with_mod_keys(MOD_CTRL_COMMAND),
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<game_actions::SetGravityTiny>::new(),
        consume.clone(),
        bindings![
            KeyCode::Digit1.with_mod_keys(MOD_CTRL_COMMAND),
            KeyCode::Numpad1.with_mod_keys(MOD_CTRL_COMMAND),
        ],
    ));

    ////////

    commands.spawn((
        include.clone(),
        Action::<actions::TogglePhysics>::new(),
        bindings![
            KeyCode::ScrollLock,
        ],
    ));
    commands.spawn((
        include.clone(),
        Action::<actions::TogglePhysicsGizmos>::new(),
        consume.clone(),
        bindings![
            KeyCode::ScrollLock.with_mod_keys(MOD_CTRL_COMMAND | ModKeys::ALT),
            KeyCode::KeyP.with_mod_keys(MOD_CTRL_COMMAND | ModKeys::ALT),
        ],
    ));

    commands.spawn((
        include.clone(),
        Action::<actions::ChangeCamera>::new(),
        consume.clone(),
        bindings![
            KeyCode::KeyV.with_mod_keys(MOD_CTRL_COMMAND),
        ],
    ));

    commands.spawn((
        include.clone(),
        Action::<actions::Zoom>::new(),
        // Scale::splat(2.5),
        ExponentialCurve::splat(1.25),
        Negate::y(),
        Bindings::spawn((
            Spawn((
                Binding::mouse_wheel(),
                SwizzleAxis::YYY,
                default_mouse_wheel_scale(1.0)
            )),
            Bidirectional::new(KeyCode::ArrowUp, KeyCode::ArrowDown),
            Bidirectional::new(GamepadButton::RightTrigger2, GamepadButton::LeftTrigger2),
        )),
    ));

}
