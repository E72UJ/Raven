use enum_map::Enum;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(
    Default, Clone, Copy, Debug, Eq, FromPrimitive, PartialEq, Enum, Serialize, Deserialize,
)]
pub enum BlendMode {
    #[default]
    Normal = 0,
    Layer = 2,
    Multiply = 3,
    Screen = 4,
    Lighten = 5,
    Darken = 6,
    Difference = 7,
    Add = 8,
    Subtract = 9,
    Invert = 10,
    Alpha = 11,
    Erase = 12,
    Overlay = 13,
    HardLight = 14,
}

impl BlendMode {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

impl From<swf::BlendMode> for BlendMode {
    fn from(blend_mode: swf::BlendMode) -> Self {
        match blend_mode {
            swf::BlendMode::Normal => BlendMode::Normal,
            swf::BlendMode::Layer => BlendMode::Layer,
            swf::BlendMode::Multiply => BlendMode::Multiply,
            swf::BlendMode::Screen => BlendMode::Screen,
            swf::BlendMode::Lighten => BlendMode::Lighten,
            swf::BlendMode::Darken => BlendMode::Darken,
            swf::BlendMode::Difference => BlendMode::Difference,
            swf::BlendMode::Add => BlendMode::Add,
            swf::BlendMode::Subtract => BlendMode::Subtract,
            swf::BlendMode::Invert => BlendMode::Invert,
            swf::BlendMode::Alpha => BlendMode::Alpha,
            swf::BlendMode::Erase => BlendMode::Erase,
            swf::BlendMode::Overlay => BlendMode::Overlay,
            swf::BlendMode::HardLight => BlendMode::HardLight,
        }
    }
}

/// 用于导出JSON数据
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Filter {
    DropShadowFilter(DropShadowFilter),
    BlurFilter(BlurFilter),
    GlowFilter(GlowFilter),
    BevelFilter(BevelFilter),
    GradientGlowFilter(GradientFilter),
    ConvolutionFilter(ConvolutionFilter),
    ColorMatrixFilter(ColorMatrixFilter),
    GradientBevelFilter(GradientFilter),
}

impl From<&swf::Filter> for Filter {
    fn from(value: &swf::Filter) -> Self {
        match value {
            swf::Filter::DropShadowFilter(filter) => {
                Filter::DropShadowFilter(filter.as_ref().to_owned().into())
            }
            swf::Filter::BlurFilter(filter) => {
                Filter::BlurFilter(filter.as_ref().to_owned().into())
            }
            swf::Filter::GlowFilter(filter) => {
                Filter::GlowFilter(filter.as_ref().to_owned().into())
            }
            swf::Filter::BevelFilter(filter) => {
                Filter::BevelFilter(filter.as_ref().to_owned().into())
            }
            swf::Filter::GradientGlowFilter(filter) => {
                Filter::GradientGlowFilter(filter.as_ref().to_owned().into())
            }
            swf::Filter::ConvolutionFilter(filter) => {
                Filter::ConvolutionFilter(filter.as_ref().to_owned().into())
            }
            swf::Filter::ColorMatrixFilter(filter) => {
                Filter::ColorMatrixFilter(filter.as_ref().to_owned().into())
            }
            swf::Filter::GradientBevelFilter(filter) => {
                Filter::GradientBevelFilter(filter.as_ref().to_owned().into())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DropShadowFilter {
    pub color: [u8; 4],
    pub blur_x: f32,
    pub blur_y: f32,
    pub angle: f32,
    pub distance: f32,
    pub strength: f32,
    pub flags: u8,
}

impl From<swf::DropShadowFilter> for DropShadowFilter {
    fn from(filter: swf::DropShadowFilter) -> Self {
        Self {
            color: [
                filter.color.r,
                filter.color.g,
                filter.color.b,
                filter.color.a,
            ],
            blur_x: filter.blur_x.to_f32(),
            blur_y: filter.blur_y.to_f32(),
            angle: filter.angle.to_f32(),
            distance: filter.distance.to_f32(),
            strength: filter.strength.to_f32(),
            flags: filter.flags.bits(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlurFilter {
    pub blur_x: f32,
    pub blur_y: f32,
    pub flags: u8,
}

impl From<swf::BlurFilter> for BlurFilter {
    fn from(filter: swf::BlurFilter) -> Self {
        Self {
            blur_x: filter.blur_x.to_f32(),
            blur_y: filter.blur_y.to_f32(),
            flags: filter.flags.bits(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlowFilter {
    pub color: [u8; 4],
    pub blur_x: f32,
    pub blur_y: f32,
    pub strength: f32,
    pub flags: u8,
}

impl From<swf::GlowFilter> for GlowFilter {
    fn from(filter: swf::GlowFilter) -> Self {
        Self {
            color: [
                filter.color.r,
                filter.color.g,
                filter.color.b,
                filter.color.a,
            ],
            blur_x: filter.blur_x.to_f32(),
            blur_y: filter.blur_y.to_f32(),
            strength: filter.strength.to_f32(),
            flags: filter.flags.bits(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BevelFilter {
    pub shadow_color: [u8; 4],
    pub highlight_color: [u8; 4],
    pub blur_x: f32,
    pub blur_y: f32,
    pub angle: f32,
    pub distance: f32,
    pub strength: f32,
    pub flags: u8,
}

impl From<swf::BevelFilter> for BevelFilter {
    fn from(filter: swf::BevelFilter) -> Self {
        Self {
            shadow_color: [
                filter.shadow_color.r,
                filter.shadow_color.g,
                filter.shadow_color.b,
                filter.shadow_color.a,
            ],
            highlight_color: [
                filter.highlight_color.r,
                filter.highlight_color.g,
                filter.highlight_color.b,
                filter.highlight_color.a,
            ],
            blur_x: filter.blur_x.to_f32(),
            blur_y: filter.blur_y.to_f32(),
            angle: filter.angle.to_f32(),
            distance: filter.distance.to_f32(),
            strength: filter.strength.to_f32(),
            flags: filter.flags.bits(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientFilter {
    pub colors: Vec<GradientRecord>,
    pub blur_x: f32,
    pub blur_y: f32,
    pub angle: f32,
    pub distance: f32,
    pub strength: f32,
    pub flags: u8,
}

impl From<swf::GradientFilter> for GradientFilter {
    fn from(filter: swf::GradientFilter) -> Self {
        Self {
            colors: filter
                .colors
                .into_iter()
                .map(|color| GradientRecord {
                    ratio: color.ratio,
                    color: [color.color.r, color.color.g, color.color.b, color.color.a],
                })
                .collect(),
            blur_x: filter.blur_x.to_f32(),
            blur_y: filter.blur_y.to_f32(),
            angle: filter.angle.to_f32(),
            distance: filter.distance.to_f32(),
            strength: filter.strength.to_f32(),
            flags: filter.flags.bits(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GradientRecord {
    pub ratio: u8,
    pub color: [u8; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConvolutionFilter {
    pub num_matrix_rows: u8,
    pub num_matrix_columns: u8,
    pub matrix: Vec<f32>,
    pub divisor: f32,
    pub bias: f32,
    pub default_color: [u8; 4],
    pub flags: u8,
}

impl From<swf::ConvolutionFilter> for ConvolutionFilter {
    fn from(filter: swf::ConvolutionFilter) -> Self {
        Self {
            num_matrix_rows: filter.num_matrix_rows,
            num_matrix_columns: filter.num_matrix_cols,
            matrix: filter.matrix,
            divisor: filter.divisor,
            bias: filter.bias,
            default_color: [
                filter.default_color.r,
                filter.default_color.g,
                filter.default_color.b,
                filter.default_color.a,
            ],
            flags: filter.flags.bits(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColorMatrixFilter {
    pub matrix: [f32; 20],
}

impl From<swf::ColorMatrixFilter> for ColorMatrixFilter {
    fn from(filter: swf::ColorMatrixFilter) -> Self {
        Self {
            matrix: filter.matrix,
        }
    }
}
