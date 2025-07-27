struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
struct Gradient {
    focal_point: f32,
    interpolation: i32,
    shape: i32,
    repeat: i32,
};
struct VertexInput {
    @location(0) position: vec2<f32>,
};
struct TextureTransforms {
    texture_matrix: mat4x4<f32>,
}
struct ViewMatrix {
    matrix: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> view_matrix: ViewMatrix;
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;
@group(1) @binding(2) var<uniform> texture_transforms: TextureTransforms;
@group(1) @binding(3) var<uniform> gradient: Gradient;
@group(1) @binding(4) var<uniform> transform: mat4x4<f32>;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let matrix_ = texture_transforms.texture_matrix;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = view_matrix.matrix * transform * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

fn find_t(uv: vec2<f32>) -> f32 {
    if gradient.shape == 1 {
        // linear
        return uv.x;
    } if gradient.shape == 2 {
        // radial
        return length(uv * 2.0 - 1.0);
    } else {
        // focal
        let uv = uv * 2.0 - 1.0;
        var d: vec2<f32> = vec2<f32>(gradient.focal_point, 0.0) - uv;
        let l = length(d);
        d = d / l;
        return l / (sqrt(1.0 - gradient.focal_point * gradient.focal_point * d.y * d.y) + gradient.focal_point * d.x);
    }
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {

    // Calculate normalized `t` position in gradient, [0.0, 1.0] being the bounds of the ratios.
    var t: f32 = find_t(in.uv);

    if gradient.repeat == 1 {
        // Pad
        t = saturate(t);
    } else if gradient.repeat == 2 {
        // Reflect
        if t < 0.0 {
            t = -t;
        }
        if (i32(t) & 1) == 0 {
            t = fract(t);
        } else {
            t = 1.0 - fract(t);
        }
    } else if gradient.repeat == 3 {
        // Repeat
        t = fract(t);
    }

    var color = textureSample(texture, texture_sampler, vec2<f32>(t, 0.0));
    if gradient.interpolation != 0 {
        color = common__linear_to_srgb(color);
    }
    // let out = color;
    // let alpha = saturate(out.a);
    // return vec4<f32>(out.rgb * alpha, alpha);
    return color;
}

/// Converts a color from linear to sRGB color space.
fn common__linear_to_srgb(linear_: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = linear_.rgb;
    if linear_.a > 0.0 {
        rgb = rgb / linear_.a;
    }
    let a = 12.92 * rgb;
    let b = 1.055 * pow(rgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), rgb);
    return vec4<f32>(mix(a, b, c) * linear_.a, linear_.a);
}
