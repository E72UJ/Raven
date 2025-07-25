use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (load_styles, setup_ui).chain())
        .add_systems(Update, button_interaction)
        .run();
}

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

#[derive(Component)]
struct DialogBox;

#[derive(Component)]
struct CloseButton;

fn load_styles(mut commands: Commands) {
    match UiStyleSheet::load_from_file("assets/style.yaml") {
        Ok(stylesheet) => {
            println!("样式表加载成功！");
            stylesheet.debug_print();
            commands.insert_resource(stylesheet);
            
            
        }
        Err(e) => {
            println!("加载样式表失败: {}", e);
            commands.insert_resource(UiStyleSheet::default());
        }
    }
}

fn setup_ui(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    stylesheet: Res<UiStyleSheet>
) {
    // 创建摄像机
    commands.spawn(Camera2d);

    // 创建对话框
    let dialog_box_entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(50.0),
                right: Val::Px(50.0),
                height: Val::Px(200.0),
                padding: stylesheet.get_padding("dialog_box"),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(stylesheet.get_background_color("dialog_box")),
            DialogBox,
        ))
        .with_children(|parent| {
            // 文本框
            parent.spawn((
                Text::new("这是一个使用YAML样式的对话框！\n点击关闭按钮来关闭对话框。"),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: stylesheet.get_font_size("textbox"),
                    ..default()
                },
                TextColor(stylesheet.get_text_color("textbox")),
                Node {
                    margin: stylesheet.get_margin("textbox"),
                    flex_grow: 1.0,
                    ..default()
                },
            ));

            // 关闭按钮
            parent.spawn((
                Button,
                Node {
                    padding: stylesheet.get_padding("button"),
                    align_self: AlignSelf::FlexEnd,
                    ..default()
                },
                BackgroundColor(stylesheet.get_background_color("button")),
                CloseButton,
            )).with_children(|button_parent| {
                button_parent.spawn((
                    Text::new("关闭"),
                    TextFont {
                        font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                        font_size: stylesheet.get_font_size("button"),
                        ..default()
                    },
                    TextColor(stylesheet.get_text_color("textbox")),
                ));
            });
        })
        .id();

    println!("对话框实体ID: {:?}", dialog_box_entity);
}

fn button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CloseButton>),
    >,
    dialog_query: Query<Entity, With<DialogBox>>,
    stylesheet: Res<UiStyleSheet>,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // 关闭对话框
                for dialog_entity in &dialog_query {
                    commands.entity(dialog_entity).despawn_recursive();
                }
                println!("对话框已关闭！");
            }
            Interaction::Hovered => {
                *background_color = Color::srgba(0.3, 0.6, 0.9, 1.0).into();
            }
            Interaction::None => {
                *background_color = stylesheet.get_background_color("button").into();
            }
        }
    }
}