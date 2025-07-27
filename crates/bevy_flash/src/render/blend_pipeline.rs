use enum_map::Enum;
use flash_runtime::parser::types::BlendMode;

use super::material::BlendMaterialKey;

#[derive(Enum, Debug, Copy, Clone)]
pub enum TrivialBlend {
    Normal,
    Add,
    Subtract,
    Screen,
    Lighten,
    Darken,
    Multiply,
}

#[derive(Enum, Debug, Copy, Clone)]
pub enum ComplexBlend {
    // Multiply,   // Can't be trivial, 0 alpha is special case
    Difference, // Can't be trivial, relies on abs operation
    Invert,     // May be trivial using a constant? Hard because it's without premultiplied alpha
    Alpha,      // Can't be trivial, requires layer tracking
    Erase,      // Can't be trivial, requires layer tracking
    Overlay,    // Can't be trivial, big math expression
    HardLight,  // Can't be trivial, big math expression
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum BlendType {
    Trivial(TrivialBlend),
    Complex(ComplexBlend),
}

impl BlendType {
    pub fn from(mode: BlendMode) -> BlendType {
        match mode {
            BlendMode::Normal => BlendType::Trivial(TrivialBlend::Normal),
            BlendMode::Layer => BlendType::Trivial(TrivialBlend::Normal),
            BlendMode::Add => BlendType::Trivial(TrivialBlend::Add),
            BlendMode::Subtract => BlendType::Trivial(TrivialBlend::Subtract),
            BlendMode::Screen => BlendType::Trivial(TrivialBlend::Screen),
            BlendMode::Lighten => BlendType::Trivial(TrivialBlend::Lighten),
            BlendMode::Darken => BlendType::Trivial(TrivialBlend::Darken),
            BlendMode::Multiply => BlendType::Trivial(TrivialBlend::Multiply),
            BlendMode::Alpha => BlendType::Complex(ComplexBlend::Alpha),
            BlendMode::Difference => BlendType::Complex(ComplexBlend::Difference),
            BlendMode::Invert => BlendType::Complex(ComplexBlend::Invert),
            BlendMode::Erase => BlendType::Complex(ComplexBlend::Erase),
            BlendMode::Overlay => BlendType::Complex(ComplexBlend::Overlay),
            BlendMode::HardLight => BlendType::Complex(ComplexBlend::HardLight),
        }
    }
}

impl From<BlendType> for BlendMaterialKey {
    fn from(value: BlendType) -> Self {
        match value {
            BlendType::Trivial(TrivialBlend::Normal) => BlendMaterialKey::NORMAL,
            BlendType::Trivial(TrivialBlend::Add) => BlendMaterialKey::BLEND_ADD,
            BlendType::Trivial(TrivialBlend::Subtract) => BlendMaterialKey::BLEND_SUBTRACT,
            BlendType::Trivial(TrivialBlend::Screen) => BlendMaterialKey::BLEND_SCREEN,
            BlendType::Trivial(TrivialBlend::Lighten) => BlendMaterialKey::BLEND_LIGHTEN,
            BlendType::Trivial(TrivialBlend::Multiply) => BlendMaterialKey::BLEND_MULTIPLY,
            BlendType::Trivial(TrivialBlend::Darken) => BlendMaterialKey::BLEND_DARKEN,
            _ => BlendMaterialKey::NORMAL,
        }
    }
}
