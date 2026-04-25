%on_update = NEW_BLOCK `update_cb, (ctx, ent, ...) {
    %ctx = GET_ARG 0, false, Struct
    %ent = GET_ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    %x = MOD %ent, 11
    %z = MOD %ent, 13

    %y = MOD %ent, 3
    %y = ADD %y, 5.0

    %pos = NEW_STRUCT [x: %x, y: %y, z: %z]
    %size = NEW_STRUCT [x: 1, y: 0.0625, z: 1]

    %rr = ADD %x, %step
    %r = DIV %rr, 11.0
    %r = MOD %r, 5.0

    %gg = ADD %z, %step
    %g = DIV %gg, 11.0
    %g = MOD %g, 5.0

    %bb = SUB %step, 17.0
    %b = DIV %z, 13.0
    %b = MOD %b, 5.0

    // Color::Srgba(Srgba(red, green, blue))
    //%srgba = NEW_STRUCT [red: %r, green: %g, blue: %b]
    //%color = NEW_SEQUENCE [%srgba]
    %color_enum = NEW_SEQUENCE [`Srgba, `red, %r, `green, %g, `blue, %b]

    %struct = NEW_STRUCT [
        offset: %pos,
        size: %size,
        color: %color_enum,
        mass: 50.0,
        //is_static: true,
        has_synth: true,
    ]
    %ent = CALL_INTRINSIC `bevy_call_func, [`spawn_cube, %struct]
    //CALL_INTRINSIC `delay, [0.01]
    //CALL_INTRINSIC `print, [%new_ent, "\n"]

    //%chv2 = NEW_SEQUENCE [1]
    %chv = NEW_SEQUENCE [`Voice, 1]
    //%ch = NEW_SEQUENCE [%chv]

    %p = MOD %step, 125
    %v_e = NEW_SEQUENCE [`Voice, 1]
    %seq = NEW_SEQUENCE [`ProgramChange, %v_e, %p]
    CALL_INTRINSIC `bevy_call_func, [`send_midi_message, %ent, %seq]

    %n = MOD %step, 100
    %n: CntF = ADD %n, 10
    %note = NEW_SEQUENCE [`Midi, %n]
    %seq = NEW_SEQUENCE [`NoteOn, %v_e, %note, 1.0]
    CALL_INTRINSIC `bevy_call_func, [`send_midi_message, %ent, %seq]

    //%noteoff_e = NEW_SEQUENCE [%chv, %note]
    %seq = NEW_SEQUENCE [`NoteOff, %v_e, %note]
    CALL_INTRINSIC `bevy_call_func, [`send_midi_message, %ent, %seq, 1.5]

    CONSUME_FUEL 10
    CALL_INTRINSIC `delay, [1.0]

    RETURN nil
}

///////

    %module = NEW_STRUCT [
        `block_size: 0.375,
        `half_side_length: 1,
        `on_update: %on_update,
        `boom_mass: 50000.0,
        `block_mass: 500.0,
        `static: false,
    ]
    CONSUME_FUEL 3
    RETURN %module
