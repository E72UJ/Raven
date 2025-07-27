use bevy::{
    asset::{Asset, Handle},
    math::{Mat4, Vec3, Vec4},
    prelude::Image,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, ShaderType,
    },
    sprite::{AlphaMode2d, Material2d},
};
use bytemuck::{Pod, Zeroable};

use flash_runtime::parser::parse_shape::{shape_utils::GradientType, tessellator::Gradient};
use swf::GradientSpread;
use swf_macro::SwfMaterial;

use super::{
    BITMAP_MATERIAL_SHADER_HANDLE, GRADIENT_MATERIAL_SHADER_HANDLE,
    SWF_COLOR_MATERIAL_SHADER_HANDLE,
};

pub trait SwfMaterial: AsBindGroup + TypePath + Asset + Material2d + Clone {
    fn update_swf_material(&mut self, swf_transform: SwfTransform);
    fn set_blend_key(&mut self, blend_key: BlendMaterialKey);
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BlendMaterialKey:u8 {
        const NORMAL                            = 0;
        const BLEND_ADD                         = 1 << 0;  // Additive blending
        const BLEND_SUBTRACT                    = 1 << 1;  // Subtractive blending
        const BLEND_SCREEN                      = 1 << 2;  // Screen blending
        const BLEND_LIGHTEN                     = 1 << 3;  // Lighten blending
        const BLEND_DARKEN                      = 1 << 4;  // Darken blending
        const BLEND_MULTIPLY                    = 1 << 5;
    }
}

impl From<&SwfColorMaterial> for BlendMaterialKey {
    fn from(value: &SwfColorMaterial) -> Self {
        value.blend_key
    }
}

impl From<&GradientMaterial> for BlendMaterialKey {
    fn from(value: &GradientMaterial) -> Self {
        value.blend_key
    }
}

impl From<&BitmapMaterial> for BlendMaterialKey {
    fn from(value: &BitmapMaterial) -> Self {
        value.blend_key
    }
}

macro_rules! material2d {
    ($name:ident, $shader:expr) => {
        impl Material2d for $name {
            fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
                $shader.into()
            }
            fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
                $shader.into()
            }
            fn alpha_mode(&self) -> AlphaMode2d {
                AlphaMode2d::Blend
            }

            fn specialize(
                descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
                _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
                key: bevy::sprite::Material2dKey<Self>,
            ) -> bevy::ecs::error::Result<
                (),
                bevy::render::render_resource::SpecializedMeshPipelineError,
            > {
                if let Some(fragment) = &mut descriptor.fragment {
                    if let Some(target) = &mut fragment.targets[0] {
                        if key.bind_group_data.contains(BlendMaterialKey::BLEND_ADD) {
                            target.blend = Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::One,
                                    dst_factor: BlendFactor::One,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent::OVER,
                            });
                        } else if key
                            .bind_group_data
                            .contains(BlendMaterialKey::BLEND_MULTIPLY)
                        {
                            target.blend = Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::Dst,
                                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent::OVER,
                            });
                        } else if key
                            .bind_group_data
                            .contains(BlendMaterialKey::BLEND_SUBTRACT)
                        {
                            target.blend = Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::One,
                                    dst_factor: BlendFactor::One,
                                    operation: BlendOperation::ReverseSubtract,
                                },
                                alpha: BlendComponent::OVER,
                            });
                        } else if key.bind_group_data.contains(BlendMaterialKey::BLEND_SCREEN) {
                            target.blend = Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::One,
                                    dst_factor: BlendFactor::OneMinusSrc,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent::OVER,
                            });
                        } else if key
                            .bind_group_data
                            .contains(BlendMaterialKey::BLEND_LIGHTEN)
                        {
                            target.blend = Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::One,
                                    dst_factor: BlendFactor::One,
                                    operation: BlendOperation::Max,
                                },
                                alpha: BlendComponent::OVER,
                            });
                        } else if key.bind_group_data.contains(BlendMaterialKey::BLEND_DARKEN) {
                            target.blend = Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::One,
                                    dst_factor: BlendFactor::One,
                                    operation: BlendOperation::Min,
                                },
                                alpha: BlendComponent::OVER,
                            });
                        }
                    }
                }

                Ok(())
            }
        }
    };
}

#[derive(AsBindGroup, TypePath, Asset, Debug, Clone, Default, SwfMaterial)]
#[bind_group_data(BlendMaterialKey)]
pub struct GradientMaterial {
    #[uniform(0)]
    pub gradient: GradientUniforms,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    #[uniform(3)]
    pub texture_transform: Mat4,
    #[uniform(4)]
    pub transform: SwfTransform,
    pub blend_key: BlendMaterialKey,
}

material2d!(GradientMaterial, GRADIENT_MATERIAL_SHADER_HANDLE);

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ShaderType, Pod, Zeroable)]
pub struct GradientUniforms {
    pub focal_point: f32,
    pub interpolation: i32,
    pub shape: i32,
    pub repeat: i32,
}
impl From<Gradient> for GradientUniforms {
    fn from(gradient: Gradient) -> Self {
        Self {
            focal_point: gradient.focal_point.to_f32().clamp(-0.98, 0.98),
            interpolation: (gradient.interpolation == swf::GradientInterpolation::LinearRgb) as i32,
            shape: match gradient.gradient_type {
                GradientType::Linear => 1,
                GradientType::Radial => 2,
                GradientType::Focal => 3,
            },
            repeat: match gradient.repeat_mode {
                GradientSpread::Pad => 1,
                GradientSpread::Reflect => 2,
                GradientSpread::Repeat => 3,
            },
        }
    }
}

#[derive(AsBindGroup, TypePath, Asset, Debug, Clone, Default, SwfMaterial)]
#[bind_group_data(BlendMaterialKey)]
pub struct SwfColorMaterial {
    #[uniform(0)]
    pub transform: SwfTransform,
    pub blend_key: BlendMaterialKey,
}

material2d!(SwfColorMaterial, SWF_COLOR_MATERIAL_SHADER_HANDLE);

#[derive(AsBindGroup, TypePath, Asset, Debug, Clone, Default, SwfMaterial)]
#[bind_group_data(BlendMaterialKey)]
pub struct BitmapMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub texture_transform: Mat4,
    #[uniform(3)]
    pub transform: SwfTransform,
    pub blend_key: BlendMaterialKey,
}

material2d!(BitmapMaterial, BITMAP_MATERIAL_SHADER_HANDLE);

#[derive(Debug, Clone, Default, ShaderType)]
pub struct SwfTransform {
    pub world_transform: Mat4,
    pub mult_color: Vec4,
    pub add_color: Vec4,
}

impl SwfTransform {
    pub fn scale(&self) -> Vec3 {
        Vec3::new(
            self.world_transform.x_axis.x,
            self.world_transform.y_axis.y,
            1.0,
        )
    }
}
