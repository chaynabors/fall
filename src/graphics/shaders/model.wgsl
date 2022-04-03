struct Globals {
    proj: mat4x4<f32>;
    proj_inv: mat4x4<f32>;
    view: mat4x4<f32>;
    view_proj: mat4x4<f32>;
    cam_pos: vec4<f32>;
};

struct VertexOutput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_coord: vec2<f32>,
    [[location(3)]] model_0: vec4<f32>,
    [[location(4)]] model_1: vec4<f32>,
    [[location(5)]] model_2: vec4<f32>,
    [[location(6)]] model_3: vec4<f32>,
    [[location(7)]] normal_0: vec3<f32>,
    [[location(8)]] normal_1: vec3<f32>,
    [[location(9)]] normal_2: vec3<f32>,
) -> VertexOutput {
    let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
    let normal_matrix = mat3x3<f32>(normal_0, normal_1, normal_2);
    let position = model * vec4<f32>(position, 1.0);

    var out: VertexOutput;
    out.position = position.xyz;
    out.normal = normal_matrix * normal;
    out.clip_position = globals.view_proj * position;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let light_position = vec3<f32>(0.0, 10.0, 0.0);
    let light_dir = normalize(light_position - in.position);
    let light = vec3<f32>(max(dot(in.normal, light_dir), 0.0) + 0.05);
    return vec4<f32>(light, 1.0);
}
