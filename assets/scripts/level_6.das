%on_update = NEW_BLOCK `on_update, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %step: CntF = LOAD %ctx, `tick_count, true

    RETURN nil
}

    %module = NEW_STRUCT [
        `on_update: %on_update,
        `boom_mass: 25000.0,
    ]
    RETURN %module
