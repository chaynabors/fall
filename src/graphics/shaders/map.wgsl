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

struct Locals {
    normal: vec3<f32>;
    x_offset: f32;
    y_offset: f32;
    rotation: f32;
    x_scale: f32;
    y_scale: f32;
};

struct VertexOutput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(0), binding(1)]]
var<uniform> locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
) -> VertexOutput {
    let position = vec4<f32>(position / 16.0, 1.0);

    var out: VertexOutput;
    out.position = position.xyz;
    out.normal = (vec4<f32>(locals.normal, 1.0)).xyz;
    out.clip_position = globals.view_proj * position;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let light_position = vec3<f32>(0.0, 1000.0, 0.0);
    let light_dir = normalize(light_position - in.position);
    let light = vec3<f32>(max(dot(in.normal, light_dir), 0.0) + 0.05);
    return vec4<f32>(in.normal, 1.0);
}
