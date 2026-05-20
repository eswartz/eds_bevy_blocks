%on_update = NEW_BLOCK `update_cb, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    CALL_INTRINSIC `delay, [0.01]
    RETURN nil
}

///////

    %module = NEW_STRUCT [
        `on_update: %on_update,
        `count: 1,
    ]
    CONSUME_FUEL 3
    RETURN %module
