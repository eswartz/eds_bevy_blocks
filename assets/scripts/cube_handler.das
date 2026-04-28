%on_update = NEW_BLOCK `on_update, (ctx, ent, ...) {
    %ctx = GET_ARG 0, false, Struct
    %ent = GET_ARG 1, false, Cnt64
    %step: CntF = LOAD %ctx, `tick_count, true

    %voice = NEW_SEQUENCE [`Voice, 1]

    %p = MOD %ent, 127

    %n = MUL %step, 17
    %n = MOD %n, 75
    %n: CntF = ADD %n, 30
    %note = NEW_SEQUENCE [`Midi, %n]

    %msg = NEW_SEQUENCE [`ProgramChange, %voice, %p]
    //CALL_INTRINSIC `bevy_call_func, [`send_midi_message, %msg]

    %msg = NEW_SEQUENCE [`NoteOn, %voice, %note, 0.125]
    //CALL_INTRINSIC `bevy_call_func, [`send_midi_message, %msg]

    %msg = NEW_SEQUENCE [`NoteOff, %voice, %note]
    //CALL_INTRINSIC `bevy_call_func, [`send_midi_message, %msg, 0.25]

    CALL_INTRINSIC `delay, [0.5]

    %y = MOD %step, 8
    %y = DIV %y, 2.0
    %vec = NEW_STRUCT [x: 0.0, y: %y, z: 0.0]

    //%pos = CALL_INTRINSIC `bevy_call_func, [`translate, %vec]
    //CALL_INTRINSIC `bevy_call_func, [`add_velocity, %vec]

    RETURN nil
}

%module = NEW_STRUCT [
    `on_update: %on_update,
]
RETURN %module
