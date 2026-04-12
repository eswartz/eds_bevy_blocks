%make_on_update = NEW_BLOCK `make_on_update, () {
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ent) {
        %ent = GET_ARG 0, false, Cnt64

        %task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        CONSUME_FUEL 1

        //CALL_INTRINSIC print, ["[", %task, " > ", %ent, ":", %new, "] "]

        // CALL_INTRINSIC print, ["[", %task, " > ", %ent, ":", %new, " "]

        CONSUME_FUEL 100

        // CALL_INTRINSIC print, [%ent, ":", %new, " "]

        // CONSUME_FUEL 1000
        // CALL_INTRINSIC print, [%ent, ":", %new, "]\n"]

        //CONSUME_FUEL 100000

        CONSUME_FUEL 10
        CONSUME_FUEL 10
        CONSUME_FUEL 10
        CONSUME_FUEL 10
        CALL_INTRINSIC spin, [1000]
        CONSUME_FUEL 1000
        CALL_INTRINSIC spin, [1000]
        CONSUME_FUEL 1000
        CALL_INTRINSIC spin, [1000]
        CONSUME_FUEL 1000
        CALL_INTRINSIC spin, [1000]
        CONSUME_FUEL 1000
        CALL_INTRINSIC spin, [1000]
        CONSUME_FUEL 1000

        CALL_INTRINSIC print, ["."]
        CONSUME_FUEL 1000

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
        `side_length: 15,
        `on_update: %on_update,
        `boom_mass: 25000.0,
        `block_mass: 500.0,
        `static: false,
    ]
    CONSUME_FUEL 3
    RETURN %module
