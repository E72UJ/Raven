struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct ViewMatrix {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> view_matrix: ViewMatrix;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let pos = view_matrix.matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, in.color);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return common__srgb_to_linear(in.color);
}

fn common__srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = srgb.rgb;
    if srgb.a > 0.0 {
        rgb = rgb / srgb.a;
    }
    let a = rgb / 12.92;
    let b = pow((rgb + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let c = step(vec3<f32>(0.04045), rgb);
    return vec4<f32>(mix(a, b, c) * srgb.a, srgb.a);
}