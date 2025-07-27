use swf::{
    BevelFilter, BevelFilterFlags, BlurFilter, BlurFilterFlags, Color, ColorMatrixFilter,
    ConvolutionFilter, ConvolutionFilterFlags, DropShadowFilter, DropShadowFilterFlags, Fixed8,
    Fixed16, GlowFilter, GlowFilterFlags, GradientFilter, GradientFilterFlags, GradientRecord,
    Rectangle, Twips,
};

use crate::parser;

/// 用于渲染的滤镜结构
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    BevelFilter(swf::BevelFilter),
    BlurFilter(swf::BlurFilter),
    ColorMatrixFilter(swf::ColorMatrixFilter),
    ConvolutionFilter(swf::ConvolutionFilter),
    DropShadowFilter(swf::DropShadowFilter),
    GlowFilter(swf::GlowFilter),
    GradientBevelFilter(swf::GradientFilter),
    GradientGlowFilter(swf::GradientFilter),
}

impl Filter {
    pub fn scale(&mut self, x: f32, y: f32) {
        match self {
            Filter::BevelFilter(filter) => filter.scale(x, y),
            Filter::BlurFilter(filter) => filter.scale(x, y),
            Filter::DropShadowFilter(filter) => filter.scale(x, y),
            Filter::GlowFilter(filter) => filter.scale(x, y),
            Filter::GradientBevelFilter(filter) => filter.scale(x, y),
            Filter::GradientGlowFilter(filter) => filter.scale(x, y),
            _ => {}
        }
    }

    pub fn calculate_dest_rect(&self, source_rect: Rectangle<Twips>) -> Rectangle<Twips> {
        match self {
            Filter::BlurFilter(filter) => filter.calculate_dest_rect(source_rect),
            Filter::GlowFilter(filter) => filter.calculate_dest_rect(source_rect),
            Filter::DropShadowFilter(filter) => filter.calculate_dest_rect(source_rect),
            Filter::BevelFilter(filter) => filter.calculate_dest_rect(source_rect),
            _ => source_rect,
        }
    }

    /// Checks if this filter is impotent.
    /// Impotent filters will have no effect if applied, and can safely be skipped.
    pub fn impotent(&self) -> bool {
        // TODO: There's more cases here, find them!
        match self {
            Filter::BlurFilter(filter) => filter.impotent(),
            Filter::ColorMatrixFilter(filter) => filter.impotent(),
            _ => false,
        }
    }
}

impl From<&swf::Filter> for Filter {
    fn from(value: &swf::Filter) -> Self {
        match value {
            swf::Filter::DropShadowFilter(filter) => {
                Filter::DropShadowFilter(filter.as_ref().to_owned())
            }
            swf::Filter::BlurFilter(filter) => Filter::BlurFilter(filter.as_ref().to_owned()),
            swf::Filter::GlowFilter(filter) => Filter::GlowFilter(filter.as_ref().to_owned()),
            swf::Filter::BevelFilter(filter) => Filter::BevelFilter(filter.as_ref().to_owned()),
            swf::Filter::GradientGlowFilter(filter) => {
                Filter::GradientGlowFilter(filter.as_ref().to_owned())
            }
            swf::Filter::ConvolutionFilter(filter) => {
                Filter::ConvolutionFilter(filter.as_ref().to_owned())
            }
            swf::Filter::ColorMatrixFilter(filter) => {
                Filter::ColorMatrixFilter(filter.as_ref().to_owned())
            }
            swf::Filter::GradientBevelFilter(filter) => {
                Filter::GradientBevelFilter(filter.as_ref().to_owned())
            }
        }
    }
}

impl From<&parser::types::Filter> for Filter {
    fn from(value: &parser::types::Filter) -> Self {
        match value {
            parser::types::Filter::DropShadowFilter(drop_shadow_filter) => {
                Filter::DropShadowFilter(DropShadowFilter {
                    color: Color {
                        r: drop_shadow_filter.color[0],
                        g: drop_shadow_filter.color[1],
                        b: drop_shadow_filter.color[2],
                        a: drop_shadow_filter.color[3],
                    },
                    blur_x: Fixed16::from_f32(drop_shadow_filter.blur_x),
                    blur_y: Fixed16::from_f32(drop_shadow_filter.blur_y),
                    angle: Fixed16::from_f32(drop_shadow_filter.angle),
                    distance: Fixed16::from_f32(drop_shadow_filter.distance),
                    strength: Fixed8::from_f32(drop_shadow_filter.strength),
                    flags: DropShadowFilterFlags::from_bits(drop_shadow_filter.flags)
                        .expect("REASON"),
                })
            }
            parser::types::Filter::BlurFilter(blur_filter) => Filter::BlurFilter(BlurFilter {
                blur_x: Fixed16::from_f32(blur_filter.blur_x),
                blur_y: Fixed16::from_f32(blur_filter.blur_y),
                flags: BlurFilterFlags::from_bits(blur_filter.flags).expect("REASON"),
            }),
            parser::types::Filter::GlowFilter(glow_filter) => Filter::GlowFilter(GlowFilter {
                color: Color {
                    r: glow_filter.color[0],
                    g: glow_filter.color[1],
                    b: glow_filter.color[2],
                    a: glow_filter.color[3],
                },
                blur_x: Fixed16::from_f32(glow_filter.blur_x),
                blur_y: Fixed16::from_f32(glow_filter.blur_y),
                strength: Fixed8::from_f32(glow_filter.strength),
                flags: GlowFilterFlags::from_bits(glow_filter.flags).expect("REASON"),
            }),
            parser::types::Filter::BevelFilter(bevel_filter) => Filter::BevelFilter(BevelFilter {
                shadow_color: Color {
                    r: bevel_filter.shadow_color[0],
                    g: bevel_filter.shadow_color[1],
                    b: bevel_filter.shadow_color[2],
                    a: bevel_filter.shadow_color[3],
                },
                highlight_color: Color {
                    r: bevel_filter.highlight_color[0],
                    g: bevel_filter.highlight_color[1],
                    b: bevel_filter.highlight_color[2],
                    a: bevel_filter.highlight_color[3],
                },
                blur_x: Fixed16::from_f32(bevel_filter.blur_x),
                blur_y: Fixed16::from_f32(bevel_filter.blur_y),
                angle: Fixed16::from_f32(bevel_filter.angle),
                distance: Fixed16::from_f32(bevel_filter.distance),
                strength: Fixed8::from_f32(bevel_filter.strength),
                flags: BevelFilterFlags::from_bits(bevel_filter.flags).expect("REASON"),
            }),
            parser::types::Filter::GradientGlowFilter(gradient_filter) => {
                Filter::GradientGlowFilter(GradientFilter {
                    colors: gradient_filter
                        .colors
                        .iter()
                        .map(|color_record| GradientRecord {
                            ratio: color_record.ratio,
                            color: Color {
                                r: color_record.color[0],
                                g: color_record.color[1],
                                b: color_record.color[2],
                                a: color_record.color[3],
                            },
                        })
                        .collect(),
                    blur_x: Fixed16::from_f32(gradient_filter.blur_x),
                    blur_y: Fixed16::from_f32(gradient_filter.blur_y),
                    angle: Fixed16::from_f32(gradient_filter.angle),
                    distance: Fixed16::from_f32(gradient_filter.distance),
                    strength: Fixed8::from_f32(gradient_filter.strength),
                    flags: GradientFilterFlags::from_bits(gradient_filter.flags).expect("REASON"),
                })
            }
            parser::types::Filter::ConvolutionFilter(convolution_filter) => {
                Filter::ConvolutionFilter(ConvolutionFilter {
                    num_matrix_rows: convolution_filter.num_matrix_rows,
                    num_matrix_cols: convolution_filter.num_matrix_rows,
                    matrix: convolution_filter.matrix.clone(),
                    divisor: convolution_filter.divisor,
                    bias: convolution_filter.bias,
                    default_color: Color {
                        r: convolution_filter.default_color[0],
                        g: convolution_filter.default_color[1],
                        b: convolution_filter.default_color[2],
                        a: convolution_filter.default_color[3],
                    },
                    flags: ConvolutionFilterFlags::from_bits(convolution_filter.flags)
                        .expect("REASON"),
                })
            }
            parser::types::Filter::ColorMatrixFilter(color_matrix_filter) => {
                Filter::ColorMatrixFilter(ColorMatrixFilter {
                    matrix: color_matrix_filter.matrix,
                })
            }
            parser::types::Filter::GradientBevelFilter(gradient_filter) => {
                Filter::GradientBevelFilter(GradientFilter {
                    colors: gradient_filter
                        .colors
                        .iter()
                        .map(|color_record| GradientRecord {
                            ratio: color_record.ratio,
                            color: Color {
                                r: color_record.color[0],
                                g: color_record.color[1],
                                b: color_record.color[2],
                                a: color_record.color[3],
                            },
                        })
                        .collect(),
                    blur_x: Fixed16::from_f32(gradient_filter.blur_x),
                    blur_y: Fixed16::from_f32(gradient_filter.blur_y),
                    angle: Fixed16::from_f32(gradient_filter.angle),
                    distance: Fixed16::from_f32(gradient_filter.distance),
                    strength: Fixed8::from_f32(gradient_filter.strength),
                    flags: GradientFilterFlags::from_bits(gradient_filter.flags).expect("REASON"),
                })
            }
        }
    }
}
