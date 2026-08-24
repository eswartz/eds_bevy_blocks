/// Huge green fields with lighted bulbs floating overhead.

%on_update = NEW_BLOCK `update_cb, (ctx: Struct, ent: Cnt64, ...) {
    %ctx = ARG 0
    %ent = ARG 1
    %step: CntF = LOAD %ctx, `tick_count, true

    %tmp = COPY %step

    %x: RealF = MOD %tmp, 15
    %tmp = DIV %tmp, 15

    //%x = DIV %x, 2
    //%x = SUB 0, %x
    %x = SUB %x, 8

    %z: RealF = MOD %tmp, 15
    %tmp = DIV %tmp, 15
    //%z = SUB 0, %z
    %z = ADD %z, 8

    %y: RealF = MOD %tmp, 15
    %tmp = DIV %tmp, 15

    %y = DIV %y, 4

    %pos = [x: %x, y: %y, z: %z]

    %sz: RealF = MOD %tmp, 31
    %tmp = DIV %tmp, 31.0

    %sz = DIV %sz, 4
    %sz: RealF = ADD %sz, 0.5

    %size = [x: %sz, y: %sz, z: %sz]

    %deg = MOD %step, 360
    %rot = [y: %deg]
    %rot: Tuple = [`Deg, %rot]

    // Make a color.
    %rr = ADD %x, %step
    %r = DIV %rr, 7.0
    %r = MOD %r, 5.0

    %gg = ADD %z, %rr
    %g = DIV %gg, 11.0
    %g = MOD %g, 5.0

    %bb = SUB %step, 17.0
    %b = DIV %z, 13.0
    %b = MOD %b, 5.0

    %color_enum = [`Srgba, `red, %r, `green, %g, `blue, %b, `alpha, 0.25]
    %r = MUL %r, 3.0
    %g = MUL %g, 3.0
    %b = MUL %b, 2.0
    %emissive = [`Srgba, `red, %r, `green, %g, `blue, %b, `alpha, 0.5]

    %shape = [`Cube, x, 0.25, y, 0.50, z, 0.75]
    %shape = [`Sphere, 0.5]

    %mass = COPY 0.0001
    %friction = COPY 0.125

    %collider_shape = [`Sphere, 1.0]
    //%collider_scale = DIV %r, 2.0
    %collider_scale = COPY %r
    %collider_scale = ADD %collider_scale, 2.0
    //%collider_scale = COPY 0.5

    %step = AND %step, 1
    IF_ZERO %step, x0f, x1f
x0f:
    %light_img = COPY "textures/mask_1.webp"
    JUMP x2f
x1f:
    %light_img = COPY "textures/mask_2.webp"
x2f:

    %intense = MUL 1000000.0, %g
    %spot_light = [
        intensity: %intense,
        range: 10.0,
        radius: 5.0,
        //outer_angle: 1.0,
        //inner_angle: 0.25,
        shadow_maps_enabled: false,
    ]
    %spot_light_tex = [
        image: %light_img,
        //cubemap_layout: `SequenceVertical,
    ]

    %intense = MUL 500000.0, %b
    %point_light = [
        intensity: %intense,
        range: 50.0,
        radius: 100.0,
        shadow_maps_enabled: false,
    ]
    %pt_light_tex = [
        image: %light_img,
        cubemap_layout: `SequenceVertical,
    ]

    %intense = MUL 1000.0, %b
    %intense = ADD %intense, 100
    %dir_light = [
        illuminance: %intense,
        shadow_maps_enabled: false,
    ]
    %dir_light_tex = [
        image: "textures/spiral_1.webp",
        tiled: false,
    ]

    %rest = [`coefficient, 1, `combine_rule, `Average]
    %grav_scale = [0.125]

    %pt_ent = SYSCALL `bevy_spawn, [
        `parent, %ent,

        // Offset is mapped to bevy_transform::components::transform::Transform.translation
        `offset, %pos,
        // Scale is mapped to bevy_transform::components::transform::Transform.scale
        `scale, %size,
        // Rotation is mapped to bevy_transform::components::transform::Transform.rotation
        `rotation, %rot,

        `rigid_body, `Dynamic,
        //`rigid_body, `Kinematic,
        `collider_shape, %collider_shape,
        `collider_scale, %collider_scale,

        `mass, %mass,
        `friction, %friction,
        `Restitution, %rest,
        `GravityScale, %grav_scale,

        `collider_shape, %shape,

        // Why not!
        //'bevy_light::spot_light::SpotLight', %spot_light,
        'SpotLight', %spot_light,
        'SpotLightTexture', %spot_light_tex,
        //'bevy_light::point_light::PointLight', %point_light,
        //'DirectionalLight', %dir_light,
        //'DirectionalLightTexture', %dir_light_tex,
        //'PointLight', %point_light,
        //'PointLightTexture', %pt_light_tex,

        `SurfaceMaterial, `Metal,
        `CollisionEventsEnabled, true,

        `Name, `LIGHT!,
    ]

    %new = SYSCALL `bevy_spawn, [
        `parent, %pt_ent,

        `color, %color_enum,
        //`normal_map, `textures/detail_normal.webp,
        //`emissive, %emissive,
        //`emissive, 1.0,   // can't see color
        //`metallic, 1.0,
        `roughness, 0.25,

        `alpha_mode, `Blend,
        `clearcoat, 0.5,

        `shape, %shape,

        //`CrosshairTargetable, nil,

        // // needed to avoid losing lights at camera edges
        // `NoFrustumCulling, nil,

        `Name, "BALL!",
    ]

    //SYSCALL `delay, [0.125]
    //SYSCALL `delay, [0.5]

    //SYSCALL `delay, [0.001]
    %vec = [x: 0, y: 10, z: 0]
    //SYSCALL `bevy_call_func, [%new, `add_velocity, %vec]

    SYSCALL `delay, [0.1]
    RETURN nil
}

///////

    %module = [
        `on_update: %on_update,
        `boom_mass: 500.0,
        `count: 1,
    ]
    RETURN %module
