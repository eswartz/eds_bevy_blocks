%make_on_update = NEW_BLOCK `make_on_update, () {
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ctx, ent, ...) {
        %ctx = ARG 0, false, Struct
        %ent = ARG 1, false, Cnt64
        %step: CntF = LOAD %ctx, `tick_count, true

//        CALL_INTRINSIC `print, ["<", %step]

        %t: CntF = COPY %ent
        //%tns: CntF = LSHR %t, 32
        //%tns2: CntF = ADD %t, %tns
        //%tn: CntF = MOD %tns2, 0xa00
        %tn: CntF = MOD %t, 0xa00
        %tnc: CntF = ADD %tn, 0x1f400
        %ts = CALL_INTRINSIC char_to_string, [%tnc]

        %task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        //CALL_INTRINSIC spin, [50]
        //CALL_INTRINSIC print, [%ts]             // uncomment for fun
        //CALL_INTRINSIC print, [%ts]

        //CALL_INTRINSIC spin, [10]
        _ = DIV 0, 0
//        CALL_INTRINSIC print, [%ts]

        //CALL_INTRINSIC `print, []
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
        `block_mass: 100.0,
        `static: false,
    ]
    RETURN %module
