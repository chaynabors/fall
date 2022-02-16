let Z_UP_TO_Y_UP = mat4x4<f32>(
    vec4<f32>(1.0, 0.0,  0.0, 0.0),
    vec4<f32>(0.0, 0.0, -1.0, 0.0),
    vec4<f32>(0.0, 1.0,  0.0, 0.0),
    vec4<f32>(0.0, 0.0,  0.0, 1.0)
);

struct Globals {
    proj: mat4x4<f32>;
    proj_inv: mat4x4<f32>;
    view: mat4x4<f32>;
    view_proj: mat4x4<f32>;
    cam_pos: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    //[[location(0)]] tex_coord: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    //@location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    //out.tex_coord = tex_coord;
    out.position = globals.view_proj * Z_UP_TO_Y_UP * vec4<f32>(position, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    //let tex = textureLoad(r_color, vec2<i32>(in.tex_coord * 256.0), 0);
    //let v = f32(tex.x) / 255.0;
    return vec4<f32>(1.0);
}
