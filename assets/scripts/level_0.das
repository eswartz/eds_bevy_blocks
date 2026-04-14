%make_on_update = NEW_BLOCK `make_on_update, () {
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ctx, ent, ...) {
        %ctx = GET_ARG 0, false, Struct
        %ent = GET_ARG 1, false, Cnt64
        %step: CntF = LOAD %ctx, `frame_step

        CALL_INTRINSIC `print, ["<", %step]

        %t = COPY %ent
        //%tns = LSHR %t, 32
        //%tns2 = ADD %t, %tns
        //%tn = MOD %tns2, 0xa00
        %tn = MOD %t, 0xa00
        %tnc = ADD %tn, 0x2000
        %ts = CALL_INTRINSIC char_to_string, [%tnc]

        CONSUME_FUEL 5
        %task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        CONSUME_FUEL 1

        CONSUME_FUEL 10
        CALL_INTRINSIC spin, [50]
        CONSUME_FUEL 50
        CONSUME_FUEL 10
        CALL_INTRINSIC print, [%ts]
        CONSUME_FUEL 10
        //CALL_INTRINSIC print, [%ts]
        //CONSUME_FUEL 10
        CONSUME_FUEL 10

        CALL_INTRINSIC spin, [10]
        CONSUME_FUEL 10

        CALL_INTRINSIC print, [%ts]
        CONSUME_FUEL 10

        CALL_INTRINSIC `print, [">"]
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
        `half_side_length: 5,
        `on_update: %on_update,
        `boom_mass: 250000.0,
        `block_mass: 500.0,
        `static: false,
    ]
    CONSUME_FUEL 3
    RETURN %module
