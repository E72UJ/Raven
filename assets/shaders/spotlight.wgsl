#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> mouse_pos: vec2<f32>;
@group(2) @binding(1) var<uniform> spotlight_radius: f32;
@group(2) @binding(2) var<uniform> edge_softness: f32;
@group(2) @binding(3) var<uniform> brightness_factor: f32;
@group(2) @binding(4) var<uniform> white_intensity: f32;
@group(2) @binding(5) var background_texture: texture_2d<f32>;
@group(2) @binding(6) var background_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // 采样背景纹理
    let background_color = textureSample(background_texture, background_sampler, mesh.uv);
    
    // 计算到鼠标位置的距离
    let distance = length((mesh.uv - mouse_pos) * vec2<f32>(1152.0, 648.0));
    
    // 计算聚光灯强度 (0.0 = 原始, 1.0 = 完全聚光灯效果)
    let spotlight_mask = 1.0 - smoothstep(spotlight_radius - edge_softness, spotlight_radius + edge_softness, distance);
    
    // 创建更白更亮的效果
    let brightened_color = background_color * brightness_factor;
    let white_boost = vec4<f32>(white_intensity, white_intensity, white_intensity, 0.0);
    let enhanced_color = brightened_color + white_boost;
    
    // 混合原图和增强后的图
    let final_color = mix(background_color, enhanced_color, spotlight_mask);
    
    return final_color;
}