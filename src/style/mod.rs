use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Resource, Deserialize, Default, Debug)]
pub struct UiStyleSheet {
    #[serde(flatten)]
    groups: HashMap<String, HashMap<String, UiStyle>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UiStyle {
    background_color: Option<[f32; 4]>,
    text_color: Option<[f32; 4]>,
    font_size: Option<f32>,
    padding: Option<[f32; 4]>,
    pub position: Option<[Option<f32>; 4]>,
    margin: Option<f32>,
    border_radius: Option<f32>,
    size: Option<[f32; 2]>,          // 新增 size 属性
    border_color: Option<[f32; 4]>,   // 新增 border_color 属性
    border_width: Option<f32>,        // 新增 border_width 属性
}

impl UiStyleSheet {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let stylesheet: UiStyleSheet = serde_yaml::from_str(&content)?;
        Ok(stylesheet)
    }

    pub fn get_style(&self, group: &str, style_name: &str) -> Option<&UiStyle> {
        self.groups.get(group)?.get(style_name)
    }

    pub fn get_background_color(&self, group: &str, style_name: &str) -> Color {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(color) = style.background_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::NONE
    }

    pub fn get_text_color(&self, group: &str, style_name: &str) -> Color {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(color) = style.text_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::WHITE
    }

    pub fn get_font_size(&self, group: &str, style_name: &str) -> f32 {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(size) = style.font_size {
                return size;
            }
        }
        14.0
    }

    pub fn get_padding(&self, group: &str, style_name: &str) -> UiRect {
        if let Some(style) = self.get_style(group, style_name) {
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

    pub fn get_position(&self, group: &str, style_name: &str) -> UiRect {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(position) = style.position {
                return UiRect {
                    left: position[3].map_or(Val::Auto, |v| Val::Px(v)),
                    right: position[1].map_or(Val::Auto, |v| Val::Px(v)),
                    top: position[0].map_or(Val::Auto, |v| Val::Px(v)),
                    bottom: position[2].map_or(Val::Auto, |v| Val::Px(v)),
                };
            }
        }
        UiRect::all(Val::Auto)
    }

    // 新增：获取尺寸
    pub fn get_size(&self, group: &str, style_name: &str) -> Option<(Val, Val)> {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(size) = style.size {
                return Some((Val::Px(size[0]), Val::Px(size[1])));
            }
        }
        None
    }

    // 新增：获取边框颜色
    pub fn get_border_color(&self, group: &str, style_name: &str) -> Color {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(color) = style.border_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::NONE
    }

    // 新增：获取边框宽度
    pub fn get_border_width(&self, group: &str, style_name: &str) -> UiRect {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(width) = style.border_width {
                return UiRect::all(Val::Px(width));
            }
        }
        UiRect::all(Val::Px(0.0))
    }

    pub fn debug_print(&self) {
        println!("=== UI样式表内容 ===");
        for (group_name, group_styles) in &self.groups {
            println!("分组: {}", group_name);
            for (style_name, style) in group_styles {
                println!("  样式名称: {}", style_name);
                
                if let Some(bg_color) = style.background_color {
                    println!("    背景色: [{}, {}, {}, {}]", bg_color[0], bg_color[1], bg_color[2], bg_color[3]);
                }
                
                if let Some(text_color) = style.text_color {
                    println!("    文字色: [{}, {}, {}, {}]", text_color[0], text_color[1], text_color[2], text_color[3]);
                }
                
                if let Some(font_size) = style.font_size {
                    println!("    字体大小: {}", font_size);
                }
                
                if let Some(size) = style.size {
                    println!("    尺寸: [宽:{}, 高:{}]", size[0], size[1]);
                }
                
                println!();
            }
        }
        println!("===================");
    }
}

// 更新加载系统
pub fn load_styles(mut commands: Commands) {
    match UiStyleSheet::load_from_file("assets/style.yaml") {
        Ok(stylesheet) => {
            stylesheet.debug_print();
            // 测试访问不同分组的样式
            let dialog_box_bg = stylesheet.get_background_color("styles", "dialog_box");
            let menu_box_bg = stylesheet.get_background_color("menu", "menu_box");
            println!("dialog_box 背景色: {:?}", dialog_box_bg);
            println!("menu_box 背景色: {:?}", menu_box_bg);
            commands.insert_resource(stylesheet);
            println!("样式表加载成功！");
        }
        Err(e) => {
            println!("加载样式表失败: {}", e);
            commands.insert_resource(UiStyleSheet::default());
        }
    }
}
pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_styles);
    }
}