%make_on_update = NEW_BLOCK `make_on_update, () {
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ctx, ent, ...) {
        %ctx = GET_ARG 0, false, Struct
        %ent = GET_ARG 1, false, CntF
        %step: CntF = LOAD %ctx, `tick_count, true

        %t = ADD %ent, %step
        %tn = MOD %t, 0xa00
        %tnc = ADD %tn, 0x1f200

        %channel: CntF = MOD %ent, 16
        %program: CntF = MOD %t, 127
        %teq: Bool = EQUAL_UINT %channel, 9
        IF_TRUE %teq, fwd, next
next:
        %program: CntF = COPY 0

fwd:
        %note = MOD %t, 0x6f
        %note = ADD %note, 0x10
        %delayi = MOD %t, 0xff     // speed here
        %delay: RealF = DIV %delayi, 256.0

        %xmsg = NEW_STRUCT [
            command: `channel_volume,
            entity: %ent,
            channel: %channel,
            velocity: 0.75,
        ]
        CALL_INTRINSIC `bevy_write_message, [`SynthMessage, %xmsg]
        %chgmsg = NEW_STRUCT [
            command: `program_change,
            entity: %ent,
            channel: %channel,
            program: %program,
        ]
        CALL_INTRINSIC `bevy_write_message, [`SynthMessage, %chgmsg]

        %onmsg = NEW_STRUCT [
            command: `note_on,
            entity: %ent,
            channel: %channel,
            note: %note,
            velocity: 1,
        ]
        CALL_INTRINSIC `bevy_write_message, [`SynthMessage, %onmsg]

        %offmsg = NEW_STRUCT [
            command: `note_off,
            entity: %ent,
            channel: %channel,
            note: %note,
            delay: %delay,
        ]
        CALL_INTRINSIC `bevy_write_message, [`SynthMessage, %offmsg]

        //CONSUME_FUEL 5
        %delay2: RealF = MUL %delay, 2.5
        CALL_INTRINSIC `delay, [%delay2]

        CONSUME_FUEL 5
        %task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        CONSUME_FUEL 1

        RETURN nil
    }

    %frame = GET_FRAME
    STORE %frame, cnt, 0

    %clos = CLOSURE %update_cb
    RETURN %clos
}

///////

    %on_update = CALL %make_on_update, []

    %module = NEW_STRUCT [
        `block_size: 0.375,
        `half_side_length: 4,
        `on_update: %on_update,
        `boom_mass: 50000.0,
        `block_mass: 500.0,
        `static: false,
    ]
    CONSUME_FUEL 3
    RETURN %module
