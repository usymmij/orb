// Vertex shader
struct CameraUniform {
    //view_proj: mat4x4<f32>,
    eye: vec3<f32>,
    aspect: f32,
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, // these get transformed
    @location(0) coord_position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
    out.coord_position = model.position;
    out.color = vec4(model.color, 1.);

    return out;
}
// Fragment shaders
    @fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sun = vec3(0., 0.707, 0.707);
    let brightness = 0.8 * (length(in.coord_position) - 0.8090169943749475);
    //return vec4(brightness + 0.6, 0.3, 0.9 - brightness, 1.);
    let lighting = dot(normalize(in.coord_position), sun) / 3. + 0.7;
    return in.color * (0.9 * brightness + 0.6) * lighting;
}
