use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Resource, Deserialize, Default)]
struct UiStyleSheet {
    styles: HashMap<String, UiStyle>,
}
#[derive(Deserialize, Clone)]
struct UiStyle {
    background_color: Option<[f32; 4]>,
    text_color: Option<[f32; 4]>,
    font_size: Option<f32>,
    padding: Option<[f32; 4]>,
    margin: Option<f32>,
    border_radius: Option<f32>,
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (load_styles).chain())
        // .add_systems(Update, button_interaction)
        .run();
}
impl UiStyleSheet {
    fn debug_print(&self) {
        println!("=== UI样式表内容 ===");
        for (style_name, style) in &self.styles {
            println!("样式名称: {}", style_name);
            
            if let Some(bg_color) = style.background_color {
                println!("  背景色: [{}, {}, {}, {}]", bg_color[0], bg_color[1], bg_color[2], bg_color[3]);
            }
            
            if let Some(text_color) = style.text_color {
                println!("  文字色: [{}, {}, {}, {}]", text_color[0], text_color[1], text_color[2], text_color[3]);
            }
            
            if let Some(font_size) = style.font_size {
                println!("  字体大小: {}", font_size);
            }
            
            if let Some(padding) = style.padding {
                println!("  内边距: [左:{}, 右:{}, 上:{}, 下:{}]", padding[0], padding[1], padding[2], padding[3]);
            }
            
            if let Some(margin) = style.margin {
                println!("  外边距: {}", margin);
            }
            
            if let Some(border_radius) = style.border_radius {
                println!("  圆角半径: {}", border_radius);
            }
            
            println!(); // 空行分隔
        }
        println!("===================");
    }
    fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let stylesheet: UiStyleSheet = serde_yaml::from_str(&content)?;
        Ok(stylesheet)
    }

    fn get_background_color(&self, style_name: &str) -> Color {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(color) = style.background_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::NONE
    }

    fn get_text_color(&self, style_name: &str) -> Color {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(color) = style.text_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::WHITE
    }

    fn get_font_size(&self, style_name: &str) -> f32 {
        if let Some(style) = self.styles.get(style_name) {
            style.font_size.unwrap_or(16.0)
        } else {
            16.0
        }
    }

    fn get_padding(&self, style_name: &str) -> UiRect {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(padding) = style.padding {
                return UiRect {
                    left: Val::Px(padding[0]),
                    right: Val::Px(padding[1]),
                    top: Val::Px(padding[2]),
                    bottom: Val::Px(padding[3]),
                };
            }
        }
        UiRect::all(Val::Px(0.0))
    }

    fn get_margin(&self, style_name: &str) -> UiRect {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(margin) = style.margin {
                return UiRect::all(Val::Px(margin));
            }
        }
        UiRect::all(Val::Px(0.0))
    }
}
fn load_styles(mut commands: Commands) {
    match UiStyleSheet::load_from_file("assets/style.yaml") {
        Ok(stylesheet) => {
            stylesheet.debug_print();
            commands.insert_resource(stylesheet);
            println!("样式表加载成功！");
        }
        Err(e) => {
            println!("加载样式表失败: {}", e);
            commands.insert_resource(UiStyleSheet::default());
        }
    }
}