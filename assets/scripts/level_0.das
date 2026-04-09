%on_update = NEW_BLOCK `on_update, (ent) {
    %ent = GET_ARG 0, false, Cnt64
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ent) {
        %ent = GET_ARG 0, false, Cnt64

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt

        CALL_INTRINSIC print, ["[", %ent, ":", %cnt, " "]

        CONSUME_FUEL 1000

        CALL_INTRINSIC print, [%ent, ":", %cnt, " "]

        CONSUME_FUEL 1000
        CALL_INTRINSIC print, [%ent, ":", %cnt, "] "]

        RETURN nil
    }

    %frame = GET_FRAME
    STORE %frame, cnt, 0

    %clos = CLOSURE %update_cb

    CALL %clos, [%ent]

    RETURN nil
}

///////

    %module = NEW_STRUCT [`side_length, 2, `on_update, %on_update, `static, false]
    CONSUME_FUEL 3
    RETURN %module
