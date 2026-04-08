%on_update = NEW_BLOCK nil, `on_update, (context) {
    CALL_INTRINSIC print, ["."]

    CONSUME_FUEL 10
    RETURN nil
}


    %module = NEW_STRUCT [`on_update, %on_update, ]
    CONSUME_FUEL 3
    RETURN %module
