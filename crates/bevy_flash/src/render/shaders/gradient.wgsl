#import bevy_sprite::{mesh2d_functions as mesh_functions, mesh2d_vertex_output::VertexOutput}
#import bevy_flash::common::{view_matrix}

struct Gradient {
    focal_point: f32,
    interpolation: i32,
    shape: i32,
    repeat: i32,
}
struct SwfTransform {
    world_matrix: mat4x4<f32>,
    mult_color: vec4<f32>,
    add_color: vec4<f32>,
}


@group(2) @binding(0) var<uniform> gradient: Gradient;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
@group(2) @binding(3) var<uniform> texture_transform: mat4x4<f32>;
@group(2) @binding(4) var<uniform> swf_transform: SwfTransform;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.uv = (mat3x3<f32>(texture_transform[0].xyz, texture_transform[1].xyz, texture_transform[2].xyz) * vec3<f32>(vertex.position.x, vertex.position.y, 1.0)).xy;
    let position: vec4<f32> = view_matrix * swf_transform.world_matrix * vec4<f32>(vertex.position, 1.0);
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        position
    );
    out.position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
    out.position.x = out.position.x - out.position.w;
    out.position.y = out.position.y + out.position.w;
    return out;
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
    let out = saturate(color * swf_transform.mult_color + swf_transform.add_color);
    let alpha = saturate(out.a);
    return vec4<f32>(out.rgb * alpha, alpha);
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
