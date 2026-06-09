// This extends the StandardMaterial with support for
// MeshTag selecting the palette index.
#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#import bevy_pbr_bindings::{base_color_texture};

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip
}
#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

// @group(#{MATERIAL_BIND_GROUP}) @binding(0) var texture: texture_2d<f32>;
// @group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_sampler: sampler;

// struct Vertex {
//     @builtin(instance_index) instance_index: u32,
//     @location(0) position: vec3<f32>,
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) world_position: vec4<f32>,
//     @location(1) color: vec4<f32>,
// };

// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
//     var out: VertexOutput;

//     var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
//     out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4(vertex.position, 1.0));
//     out.clip_position = position_world_to_clip(out.world_position.xyz);

//     // // Lookup the tag for the given mesh.
//     // var tag = i32(mesh_functions::get_tag(vertex.instance_index));
//     // if tag < 0 {
//     //     tag = -tag;
//     // }
//     // tag &= 255;

//     // var pbr_input = pbr_input_from_standard_material(in, is_front);
//     // let tex_dim = textureDimensions(pbr_input.base_color_texture);
//     // // Find the texel coordinate as derived from the tag.
//     // let texel_coord = vec2<u32>(tag % tex_dim.x, tag / tex_dim.x);

//     // out.color = textureLoad(pbr_input.base_color_texture, texel_coord, 0);
//     return out;
// }

@fragment
fn fragment(
    in_orig: VertexOutput,
    @builtin(front_facing) is_front: bool,

) -> FragmentOutput {
    // Get tag, used as iteration count (-ve = done, +ve = progress)
    var tag = i32(mesh_functions::get_tag(in_orig.instance_index));
    var in = in_orig;
/*

    let tex_dim = textureDimensions(base_color_texture);
    // Find the texel coordinate as derived from the tag.
    let texel_coord = vec2<u32>(tag % tex_dim.x, tag / tex_dim.x);
    xin.uv = vec2(f32(texel_coord.x) / f32(tex_dim.x), f32(texel_coord.y) / f32(tex_dim.y));

    // Generate a PbrInput struct from the StandardMaterial bindings.
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    //pbr_input.material.base_color.r += f32(tag & 0xf) / 16.0;
    //pbr_input.material.base_color.g += f32(tag & 0xf0) / 256.0;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // // clustered decals
    // pbr_input.material.base_color = apply_decal_base_color(
    //     in.world_position.xyz,
    //     in.position.xy,
    //     pbr_input.material.base_color
    // );

#ifdef PREPASS_PIPELINE
    // In deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;

    // Apply lighting.
    out.color = apply_pbr_lighting(pbr_input);

    // Apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr).
    // Note: this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    //out.color = vec4<f32>(f32(tag) / 256.0, 1.0, 1.0, 1.0);
*/

#ifdef PREPASS_PIPELINE
    // In deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;

    //let tex_dim = textureDimensions(base_color_texture);
    //let texel_coord = vec2<u32>(tag % tex_dim.x, tag / tex_dim.x);
    //out.color = textureLoad(base_color_texture, texel_coord, 0);

    let c = f32(tag & 0xff ^ ((tag & 0x2) >> 1)) / 256.0;
    out.color = vec4<f32>(c, c, c, 1.0);
#endif
    return out;

}
