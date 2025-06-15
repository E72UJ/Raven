#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> mouse_pos: vec2<f32>;
@group(2) @binding(1) var<uniform> glow_radius: f32;
@group(2) @binding(2) var<uniform> glow_intensity: f32;
@group(2) @binding(3) var texture: texture_2d<f32>;
@group(2) @binding(4) var texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // 获取原始纹理颜色
    var base_color = textureSample(texture, texture_sampler, mesh.uv);
    
    // 计算片段在世界坐标中的位置
    let world_pos = mesh.world_position.xy;
    
    // 计算到鼠标位置的距离
    let distance = length(world_pos - mouse_pos);
    
    // 计算发光强度（距离越近越亮）
    let glow_factor = max(0.0, 1.0 - distance / glow_radius);
    let final_glow = glow_factor * glow_intensity;
    
    // 混合金色发光效果
    let glow_color = vec3<f32>(1.0, 1.0, 1.0); // 白色
    let final_color = base_color.rgb + glow_color * final_glow * 0.5;
    
    return vec4<f32>(final_color, base_color.a);
}