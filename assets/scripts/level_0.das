%make_on_update = NEW_BLOCK `make_on_update, () {
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ctx: Struct, ent: Cnt64, ...) {
        %ctx = ARG 0
        %ent = ARG 1
        %step: CntF = LOAD %ctx, `tick_count, true

//        SYSCALL `print, ["<", %step]

        %t: CntF = CAST %ent, `coerce
        //%tns: CntF = LSHR %t, 32
        //%tns2: CntF = ADD %t, %tns
        //%tn: CntF = MOD %tns2, 0xa00
        %tn: CntF = MOD %t, 0xa00
        %tnc: CntF = ADD %tn, 0x1f400
        %ts = SYSCALL char_to_string, [%tnc]

        %task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        //SYSCALL spin, [50]
        //SYSCALL print, [%ts]             // uncomment for fun
        //SYSCALL print, [%ts]

        //SYSCALL spin, [10]
        _ = DIV 0, 0
//        SYSCALL print, [%ts]

        //SYSCALL `print, []
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
        `boom_mass: 250.0,
        `block_mass: 100.0,
        `static: false,
    ]
    RETURN %module
