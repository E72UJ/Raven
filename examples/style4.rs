use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// 组件定义
#[derive(Component)]
struct DialogBox;

#[derive(Component)]
struct DialogText;

#[derive(Component)]
struct DialogButton;

#[derive(Component)]
struct NameBox;

#[derive(Component)]
struct NameText;

#[derive(Component)]
struct SpriteBox;

// 样式表结构
#[derive(Resource, Debug, Clone, Deserialize, Serialize)]
pub struct UiStyleSheet {
    pub styles: HashMap<String, StyleConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StyleConfig {
    pub background_color: Option<[f32; 4]>,
    pub text_color: Option<[f32; 4]>,
    pub font_size: Option<f32>,
    pub padding: Option<[f32; 4]>,
    pub margin: Option<f32>,
    pub border_color: Option<[f32; 4]>,
    pub border_width: Option<f32>,
    pub position: Option<[f32; 4]>,
    pub size: Option<[f32; 2]>,
}

impl UiStyleSheet {
    pub fn get_background_color(&self, style_name: &str) -> Color {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(color) = style.background_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::srgba(0.1, 0.1, 0.1, 0.9) // 默认颜色
    }

    pub fn get_text_color(&self, style_name: &str) -> Color {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(color) = style.text_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::WHITE // 默认颜色
    }

    pub fn get_font_size(&self, style_name: &str) -> f32 {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(size) = style.font_size {
                return size;
            }
        }
        16.0 // 默认字体大小
    }

    pub fn get_padding(&self, style_name: &str) -> UiRect {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(padding) = style.padding {
                return UiRect::all(Val::Px(padding[0]));
            }
        }
        UiRect::all(Val::Px(10.0)) // 默认内边距
    }

    pub fn get_margin(&self, style_name: &str) -> f32 {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(margin) = style.margin {
                return margin;
            }
        }
        10.0 // 默认外边距
    }

    pub fn get_position(&self, style_name: &str) -> [f32; 4] {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(position) = style.position {
                return position;
            }
        }
        [0.0, 0.0, 0.0, 0.0] // 默认位置
    }

    pub fn get_size(&self, style_name: &str) -> [f32; 2] {
        if let Some(style) = self.styles.get(style_name) {
            if let Some(size) = style.size {
                return size;
            }
        }
        [100.0, 100.0] // 默认尺寸
    }
}

// 主要的设置函数
fn setup_ui(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    stylesheet: Res<UiStyleSheet>
) {
    // 创建摄像机
    commands.spawn(Camera2d);

    // 创建精灵实体
    let sprite_size = stylesheet.get_size("spritebox");
    let sprite_position = stylesheet.get_position("spritebox");
    println!("精灵位置 - x: {}, y: {}", sprite_position[0], sprite_position[1]);
    commands.spawn((
        Name::new("spritebox"),
        Transform::from_xyz(sprite_position[1], sprite_position[0], 0.0),
        Sprite {
            image: asset_server.load("characters/protagonist/default.png"),
            custom_size: Some(Vec2::new(sprite_size[0], sprite_size[1])),
            ..default()
        },
        Visibility::Visible,
        SpriteBox,
    ));

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
            // 对话文本
            parent.spawn((
                Text::new("这里是对话内容..."),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: stylesheet.get_font_size("textbox"),
                    ..default()
                },
                TextColor(stylesheet.get_text_color("textbox")),
                Node {
                    flex_grow: 1.0,
                    margin: UiRect::all(Val::Px(stylesheet.get_margin("textbox"))),
                    ..default()
                },
                DialogText,
            ));

            // 按钮容器
            parent
                .spawn(Node {
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Button,
                        Node {
                            padding: stylesheet.get_padding("button"),
                            ..default()
                        },
                        BackgroundColor(stylesheet.get_background_color("button")),
                        DialogButton,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("下一页"),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: stylesheet.get_font_size("button"),
                                ..default()
                            },
                            TextColor(stylesheet.get_text_color("button")),
                        ));
                    });
                });
        })
        .id();

    // 创建名字框
    let name_position = stylesheet.get_position("namebox");
    let name_size = stylesheet.get_size("namebox");
    
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(name_position[0]),
                left: Val::Px(name_position[1]),
                width: Val::Px(name_size[0]),
                height: Val::Px(name_size[1]),
                padding: stylesheet.get_padding("namebox"),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(stylesheet.get_background_color("namebox")),
            NameBox,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("角色名"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: stylesheet.get_font_size("namebox"),
                    ..default()
                },
                TextColor(stylesheet.get_text_color("namebox")),
                NameText,
            ));
        });
}

// 修改后的加载样式表函数
fn load_stylesheet(mut commands: Commands) {
    // 尝试从YAML文件加载样式表
    let stylesheet = match fs::read_to_string("assets/style.yaml") {
        Ok(yaml_content) => {
            match serde_yaml::from_str::<UiStyleSheet>(&yaml_content) {
                Ok(loaded_stylesheet) => {
                    println!("成功从YAML文件加载样式表");
                    loaded_stylesheet
                },
                Err(e) => {
                    println!("解析YAML文件失败: {}", e);
                    create_default_stylesheet()
                }
            }
        },
        Err(e) => {
            println!("读取YAML文件失败: {}", e);
            create_default_stylesheet()
        }
    };

    commands.insert_resource(stylesheet);
}

// 创建默认样式表的辅助函数
fn create_default_stylesheet() -> UiStyleSheet {
    let mut styles = HashMap::new();
    
    styles.insert("dialog_box".to_string(), StyleConfig {
        background_color: Some([0.1, 0.1, 0.1, 0.9]),
        padding: Some([20.0, 20.0, 20.0, 20.0]),
        border_color: Some([0.5, 0.5, 0.5, 1.0]),
        border_width: Some(2.0),
        ..Default::default()
    });

    styles.insert("textbox".to_string(), StyleConfig {
        text_color: Some([1.0, 1.0, 1.0, 1.0]),
        font_size: Some(18.0),
        margin: Some(10.0),
        ..Default::default()
    });

    styles.insert("button".to_string(), StyleConfig {
        background_color: Some([0.2, 0.5, 0.8, 1.0]),
        text_color: Some([1.0, 1.0, 1.0, 1.0]),
        font_size: Some(16.0),
        padding: Some([12.0, 24.0, 8.0, 8.0]),
        ..Default::default()
    });

    styles.insert("namebox".to_string(), StyleConfig {
        background_color: Some([0.3, 0.2, 0.5, 0.95]),
        text_color: Some([1.0, 1.0, 1.0, 1.0]),
        font_size: Some(20.0),
        position: Some([260.0, 70.0, 0.0, 0.0]),
        size: Some([150.0, 50.0]),
        border_color: Some([0.8, 0.7, 0.9, 1.0]),
        border_width: Some(1.5),
        ..Default::default()
    });

    styles.insert("spritebox".to_string(), StyleConfig {
        position: Some([50.0, 50.0, 0.0, 0.0]),
        size: Some([400.0, 600.0]),
        background_color: Some([0.4, 0.4, 0.1, 1.0]),
        ..Default::default()
    });

    UiStyleSheet { styles }
}

// 按钮交互系统
fn button_system(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<DialogButton>)>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                println!("对话按钮被点击!");
            }
            _ => {}
        }
    }
}

// 默认实现
impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            background_color: None,
            text_color: None,
            font_size: None,
            padding: None,
            margin: None,
            border_color: None,
            border_width: None,
            position: None,
            size: None,
        }
    }
}

// 应用程序主函数
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (load_stylesheet, setup_ui).chain())
        .add_systems(Update, button_system)
        .run();
}