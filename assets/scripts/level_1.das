
%make_on_update = NEW_BLOCK `make_on_update, (in_use_synths) {
    LOCAL cnt, false, CntF

    %in_use_synths = ARG 0
    LOCAL use_synths, false, Bool

    // Make it visible to closure.
    %frame = GET_FRAME
    STORE %frame, use_synths, %in_use_synths

    %update_cb = NEW_BLOCK `update_cb, (ctx: Struct, ent: Cnt64, ...) {
        %ctx = ARG 0
        %ent = ARG 1
        %step: CntF = LOAD %ctx, `tick_count, true

        // Startup delay
        %seq: Bool = LESS_U %step, 600
        IF_TRUE %seq, sleep, skip
sleep:
        %d3: RealF = MOD %ent, 127
        %d3: RealF = DIV %d3, 1.0
        SYSCALL `delay, [%d3]
skip:
        %t: CntF = ADD %ent, %step
        %tn = COPY %t
        //%tnc = ADDU %tn, 0x1f200

        %channel: CntF = MOD %tn, 16
        %program: CntF = MOD %tn, 127
        %teq: Bool = EQUAL_U %channel, 9
        IF_TRUE %teq, next, fwd
next:
        %program: CntF = COPY 0

fwd:
        %note = MOD %t, 0x6f
        %note = ADD %note, 0x10
        %delayi = MOD %t, 0x1f     // speed here
        %delay: RealF = DIV %delayi, 256.0

        %outer = GET_FRAME 1
        %use_synths = LOAD %outer, use_synths
        IF_TRUE %use_synths, add_notes, no_notes

add_notes:
        %msg = NEW_STRUCT [
            command: `channel_volume,
            entity: %ent,
            velocity: 0.5
        ]
        SYSCALL `bevy_call_func, [`write_message, `MidiSynthMessage, %msg]
        %msg = NEW_STRUCT [
            command: `program_change,
            entity: %ent,
            channel: %channel,
            program: %program,
        ]
        SYSCALL `bevy_call_func, [`write_message, `MidiSynthMessage, %msg]

        %msg = NEW_STRUCT [
            command: `note_on,
            entity: %ent,
            channel: %channel,
            midi: %note,
            velocity: 0.25,
        ]
        SYSCALL `bevy_call_func, [`write_message, `MidiSynthMessage, %msg]

        %msg = NEW_STRUCT [
            command: `note_off,
            entity: %ent,
            channel: %channel,
            midi: %note,
            delay: %delay,
        ]
        SYSCALL `bevy_call_func, [`write_message, `MidiSynthMessage, %msg]

no_notes:
        %test = SYS_CALL `random_number, [0.0, 50.0]
        //%test: CntF = SYS_CALL `round, [%test]
        %test = LESS_R %test, 10.0
        IF_TRUE %test, no_fun, hop
hop:
        %vy = SYS_CALL `random_number, [-25.0, -5.0]
        %vx = SYS_CALL `random_number, [-5.0, 5.0]
        %vz = SYS_CALL `random_number, [-5.0, 5.0]
        %vel = NEW_STRUCT [
            x: %vx,
            y: %vy,
            z: %vz,
        ]

        SYS_CALL `bevy_call_func, [%ent,
            //%ent, `insert_components,
            //`LinearVelocity, %vel,
            `add_velocity, %vel, `World,
        ]
no_fun:


        %delay2: RealF = MUL %delay, 5.0
        %delay2: RealF = MOD %delay2, 10.0
        SYSCALL `delay, [%delay2]

        //%task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        RETURN nil
    }

    %frame = GET_FRAME
    STORE %frame, cnt, 0

    %clos = CLOSURE %update_cb
    RETURN %clos
}

///////

    %use_synths = COPY true
    %on_update = CALL %make_on_update, [%use_synths]

    %module = NEW_STRUCT [
        `block_size: 0.55,
        `half_side_length: 4,
//        `half_side_length: 3,
        `on_update: %on_update,
        `boom_mass: 10000.0,
        `block_mass: 50.0,
        `static: false,
        `cube_gap: 0.02,
        `collision_margin: 0.0,
        `with_synth: %use_synths,
    ]
    RETURN %module
