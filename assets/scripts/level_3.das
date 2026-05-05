%on_update = NEW_BLOCK `update_cb, (ctx, ent, ...) {
    %ctx = GET_ARG 0, false, Struct
    %ent = GET_ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    %tmp = COPY %step

    %y: RealF = MOD %tmp, 15
    %tmp = DIV %tmp, 15

    %y = DIV %y, 4

    %x: RealF = MOD %tmp, 15
    %tmp = DIV %tmp, 15

    //%x = DIV %x, 2
    //%x = SUB 0, %x
    %x = SUB %x, 8

    %z: RealF = MOD %tmp, 15
    %tmp = DIV %tmp, 15
    //%z = SUB 0, %z
    %z = ADD %z, 8

    %pos = NEW_STRUCT [x: %x, y: %y, z: %z]

    %sz: RealF = MOD %tmp, 31
    %tmp = DIV %tmp, 31.0

    %sz = DIV %sz, 4
    %sz: RealF = ADD %sz, 0.5

    %size = NEW_STRUCT [x: %sz, y: %sz, z: %sz]

    %deg = MOD %step, 360
    %rot = NEW_STRUCT [y: %deg]
    %rot = NEW_SEQUENCE [`Deg, %rot]

    // Make a color.
    %rr = ADD %x, %step
    %r = DIV %rr, 7.0
    %r = MOD %r, 5.0

    %gg = ADD %z, %step
    %g = DIV %gg, 11.0
    %g = MOD %g, 5.0

    %bb = SUB %step, 17.0
    %b = DIV %z, 13.0
    %b = MOD %b, 5.0

    %color_enum = NEW_SEQUENCE [`Srgba, `red, %r, `green, %g, `blue, %b, `alpha, 0.25]
    %r = MUL %r, 3.0
    %g = MUL %g, 3.0
    %b = MUL %b, 2.0
    %emissive = NEW_SEQUENCE [`Srgba, `red, %r, `green, %g, `blue, %b, `alpha, 0.5]

    %shape = NEW_SEQUENCE [`Cube, x, 0.25, y, 0.50, z, 0.75]
    %shape = NEW_SEQUENCE [`Sphere, 0.5]

    %mass = COPY 0.0001
    %friction = COPY 0.0

    %collider_shape = NEW_SEQUENCE [`Sphere, 1.0]
    //%collider_scale = DIV %r, 2.0
    %collider_scale = COPY %r
    %collider_scale = ADD %collider_scale, 2.0
    //%collider_scale = COPY 0.5

    %intense = MUL 100000.0, %g
    %spot_light = NEW_STRUCT [
        intensity: %intense,
        range: 10.0,
        radius: 5.0,
        //outer_angle: 1.0,
        //inner_angle: 0.25,
        shadows_enabled: false,
    ]

    %intense = MUL 500000.0, %b
    %point_light = NEW_STRUCT [
        intensity: %intense,
        range: 50.0,
        radius: 100.0,
        shadows_enabled: false,
    ]

    %intense = MUL 500.0, %b
    %dir_light = NEW_STRUCT [
        illuminance: %intense,
        shadows_enabled: false,
    ]
    %dir_light_tex = NEW_STRUCT [
        image: "textures/mask_1.webp",
        tiled: true,
    ]
    %pt_light_tex = NEW_STRUCT [
        image: "textures/mask_2.webp",
        cubemap_layout: `SequenceVertical,
    ]
    %spot_light_tex = NEW_STRUCT [
        image: "textures/mask_2.webp",
        //cubemap_layout: `SequenceVertical,
    ]


    CALL_INTRINSIC `bevy_spawn, [
        // Offset is mapped to bevy_transform::components::transform::Transform.translation
        `offset, %pos,
        // Scale is mapped to bevy_transform::components::transform::Transform.scale
        `scale, %size,
        // Rotation is mapped to bevy_transform::components::transform::Transform.rotation
        `rotation, %rot,

        `color, %color_enum,
        `normal_map, `textures/detail_normal.webp,
        //`emissive, %emissive,
        //`emissive, 4.0,
        `metallic, 1.0,
        `roughness, 0.25,

        `alpha_mode, `Blend,
        `clearcoat, 0.5,

        `rigid_body, `Dynamic,

        `shape, %shape,
        `collider_shape, %collider_shape,
        `collider_scale, %collider_scale,

        `mass, %mass,
        `friction, %friction,

        // Why not!
        'bevy_light::spot_light::SpotLight', %spot_light,
        //'SpotLightTexture', %spot_light_tex,
        //'bevy_light::point_light::PointLight', %point_light,
        //'DirectionalLight', %dir_light,
        'PointLight', %point_light,
        'PointLightTexture', %pt_light_tex,
    ]
    //CALL_INTRINSIC `delay, [0.125]
    CALL_INTRINSIC `delay, [0.001]

    RETURN nil
}

///////

    %module = NEW_STRUCT [
        `on_update: %on_update,
        `boom_mass: 50000.0,
        `count: 1,
    ]
    CONSUME_FUEL 3
    RETURN %module
