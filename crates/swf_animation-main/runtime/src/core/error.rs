use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Animation `{0}` not found")]
    AnimationNotFound(String),

    #[error("Animation event `{0}` not found")]
    AnimationEventNotFound(String),

    #[error("skin `{0}` not found")]
    SkinNotFound(String),

    #[error("skin part `{0}` not found")]
    SkinPartNotFound(String),
}
