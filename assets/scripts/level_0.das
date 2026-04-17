%make_on_update = NEW_BLOCK `make_on_update, () {
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ctx, ent, ...) {
        %ctx = GET_ARG 0, false, Struct
        %ent = GET_ARG 1, false, Cnt64
        %step: CntF = LOAD %ctx, `frame_step, true

  //      CALL_INTRINSIC `print, ["<", %step]

        %t = COPY %ent
        //%tns = LSHR %t, 32
        //%tns2 = ADD %t, %tns
        //%tn = MOD %tns2, 0xa00
        %tn = MOD %t, 0xa00
        %tnc = ADD %tn, 0x1f400
        %ts = CALL_INTRINSIC char_to_string, [%tnc]

        %channel = MOD %t, 9

        %note = MOD %t, 0x40
        %delayi = MOD %t, 0x100
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
            program: %note,
        ]
        CALL_INTRINSIC `bevy_write_message, [`SynthMessage, %chgmsg]

        %onmsg = NEW_STRUCT [
            command: `note_on,
            entity: %ent,
            channel: %channel,
            note: %note,
            velocity: 0.5,
        ]
        CALL_INTRINSIC `bevy_write_message, [`SynthMessage, %onmsg]

        %delay2: RealF = ADD %delay, 0.125
        %offmsg = NEW_STRUCT [
            command: `note_off,
            entity: %ent,
            channel: %channel,
            note: %note,
        ]
        //CALL_INTRINSIC `bevy_write_message, [`SynthMessage, %offmsg]

        //CALL_INTRINSIC `delay, [0.125]

        CONSUME_FUEL 5
        %task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        CONSUME_FUEL 1

       // CONSUME_FUEL 500
     //CALL_INTRINSIC spin, [500]
       // CONSUME_FUEL 500
     //CALL_INTRINSIC spin, [500]
       // CONSUME_FUEL 500
     //CALL_INTRINSIC spin, [500]
        //CONSUME_FUEL 500
     //CALL_INTRINSIC spin, [500]

        //CONSUME_FUEL 50
        //CALL_INTRINSIC print, [%ts]
        //CONSUME_FUEL 10
        //CALL_INTRINSIC print, [%ts]
        //CONSUME_FUEL 10

        CALL_INTRINSIC spin, [50]
        CONSUME_FUEL 100

        //CALL_INTRINSIC print, [%ts]
        CONSUME_FUEL 10

//        CALL_INTRINSIC `print, [">"]

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
        `block_size: 0.5,
        `half_side_length: 4,
        `on_update: %on_update,
        `boom_mass: 250000.0,
        `block_mass: 500.0,
        `static: false,
    ]
    CONSUME_FUEL 3
    RETURN %module
