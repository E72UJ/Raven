#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> material: BorderMaterial;
@group(2) @binding(1) var base_texture: texture_2d<f32>;
@group(2) @binding(2) var base_sampler: sampler;

struct BorderMaterial {
    border_width: f32,
    border_color: vec4<f32>,
    inner_color_multiplier: vec4<f32>,
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let color = textureSample(base_texture, base_sampler, uv);
    
    // 如果像素是透明的，直接返回透明
    if color.a < 0.01 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    // 计算到边缘的距离（基于UV坐标）
    let border_dist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    
    // 如果在边框范围内，返回边框颜色
    if border_dist < material.border_width {
        return material.border_color;
    }
    
    // 否则返回内部颜色（可以应用颜色调制）
    return color * material.inner_color_multiplier;
}