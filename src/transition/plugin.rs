// src/transition/plugin.rs
use bevy::prelude::*;
use super::fade::{fade_system, cleanup_completed_fades};

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fade_system, cleanup_completed_fades));
    }
}