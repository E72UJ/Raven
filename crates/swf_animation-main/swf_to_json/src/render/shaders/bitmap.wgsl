struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct TextureTransforms {
    texture_matrix: mat4x4<f32>,
}
struct ViewMatrix {
    matrix: mat4x4<f32>,
}
struct VertexInput {
    /// The position of the vertex in object space.
    @location(0) position: vec2<f32>,
}

@group(0) @binding(0) var<uniform> view_matrix: ViewMatrix;
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;
@group(1) @binding(2) var<uniform> texture_transforms: TextureTransforms;
@group(1) @binding(3) var<uniform> transforms: mat4x4<f32>;

override late_saturate: bool = false;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let matrix_ = texture_transforms.texture_matrix;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = view_matrix.matrix * transforms * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(texture, texture_sampler, in.uv);
    // Texture is premultiplied by alpha.
    // Unmultiply alpha, apply color transform, remultiply alpha.
    if color.a > 0.0 {
        color = vec4<f32>(color.rgb / color.a, color.a);
        if !late_saturate {
            color = saturate(color);
        }
        color = vec4<f32>(color.rgb * color.a, color.a);
        if late_saturate {
            color = saturate(color);
        }
    }
    return color;
}