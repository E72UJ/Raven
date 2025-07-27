#import bevy_sprite::{mesh2d_functions as mesh_functions, mesh2d_vertex_output::VertexOutput}
#import bevy_flash::common::{view_matrix}


struct SwfTransform {
    world_matrix: mat4x4<f32>,
    mult_color: vec4<f32>,
    add_color: vec4<f32>,
}
@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> texture_transform: mat4x4<f32>;
@group(2) @binding(3) var<uniform> swf_transform: SwfTransform;
override late_saturate: bool = false;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.uv = (mat3x3<f32>(texture_transform[0].xyz, texture_transform[1].xyz, texture_transform[2].xyz) * vec3<f32>(vertex.position.x, vertex.position.y, 1.0)).xy;
    var position: vec4<f32> = view_matrix * swf_transform.world_matrix * vec4<f32>(vertex.position, 1.0);
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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(texture, texture_sampler, in.uv);

    if color.a > 0.0 {
        color = vec4<f32>(color.rgb / color.a, color.a);
        color = color * swf_transform.mult_color + swf_transform.add_color;
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
