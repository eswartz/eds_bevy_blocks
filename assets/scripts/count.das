%on_update = NEW_BLOCK nil, `on_update, (context) {
    %context = GET_ARG 0, false, Cnt64
    CALL_INTRINSIC print, [%context, "\n"]

    CONSUME_FUEL 100
    RETURN nil
}

///////

    %module = NEW_STRUCT [`side_length, 2, `on_update, %on_update,]
    CONSUME_FUEL 3
    RETURN %module
