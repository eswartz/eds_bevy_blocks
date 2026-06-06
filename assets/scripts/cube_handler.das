%on_update = NEW_BLOCK `on_update, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %step: CntF = LOAD %ctx, `tick_count, true

    %voice = COPY 1

    %p = MOD %ent, 127

    %n = MUL %step, 17
    %n = MOD %n, 75
    %n: CntF = ADD %n, 20

    %msg = NEW_STRUCT [
        command: `program_change,
        entity: %ent,
        channel: %voice,
        program: %p,
    ]
    SYSCALL `bevy_call_func, [`write_message, `MidiSynthMessage, %msg]

    %msg = NEW_STRUCT [
        command: `note_on,
        entity: %ent,
        channel: %voice,
        midi: %n,
        drums: true,
        velocity: 0.125,
    ]
    SYSCALL `bevy_call_func, [`write_message, `MidiSynthMessage, %msg]

    %msg = NEW_STRUCT [
        command: `note_off,
        entity: %ent,
        channel: %voice,
        midi: %n,
        drums: true,
        delay: 0.25,
    ]
    SYSCALL `bevy_call_func, [`write_message, `MidiSynthMessage, %msg]

    SYSCALL `delay, [0.125]

    %y = MOD %step, 8
    %y = DIV %y, 2.0
    %vec = NEW_STRUCT [x: -0.1, y: %y, z: 0.1]

    //%pos = SYSCALL `bevy_call_func, [%ent, `translate, %vec]
    //SYSCALL `bevy_call_func, [%ent, `add_velocity, %vec]

    RETURN nil
}

%module = NEW_STRUCT [
    `on_update: %on_update,
]
RETURN %module
