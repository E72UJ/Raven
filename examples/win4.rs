use bevy::prelude::*;
use std::process::Command;

// 链接按钮组件
#[derive(Component)]
struct LinkButton {
    url: String,
}

// 跨平台打开URL的函数
fn open_url(url: &str) -> Result<(), String> {
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(url)
            .spawn()
    } else {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
    };

    match result {
        Ok(_) => {
            println!("成功打开链接: {}", url);
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("打开链接失败: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// 处理链接点击的系统
fn handle_link_clicks(
    mut interaction_query: Query<
        (
            &Interaction,
            &LinkButton,
            &mut BackgroundColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, link_button, mut bg_color, children) in &mut interaction_query {
        if let Ok(mut text_color) = text_query.get_mut(children[0]) {
            match *interaction {
                Interaction::Pressed => {
                    if let Err(e) = open_url(&link_button.url) {
                        println!("打开链接时出错: {}", e);
                    }
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.6, 0.2));
                    text_color.0 = Color::srgb(1.0, 1.0, 1.0);
                }
                Interaction::Hovered => {
                    *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
                    text_color.0 = Color::srgb(1.0, 1.0, 1.0);
                }
                Interaction::None => {
                    *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
                    text_color.0 = Color::srgb(0.9, 0.9, 0.9);
                }
            }
        }
    }
}

// 游戏场景设置
fn setup_game_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 添加摄像机
    commands.spawn(Camera2d);

    // 创建主容器
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }).with_children(|parent| {
        // 创建列表容器
        parent.spawn(Node {
            width: Val::Px(300.0),
            height: Val::Auto,
            flex_direction: FlexDirection::Column,
            ..default()
        }).with_children(|list| {
            // 标题
            list.spawn((
                Text::new("🔗 链接列表"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 链接项目数据
            let links = [
                ("🌐 官方网站", "https://example.com"),
                ("📦 GitHub", "https://github.com"),
                ("📧 邮箱", "mailto:test@example.com"),
                ("📚 文档", "https://docs.rs"),
                ("💬 社区", "https://discord.com"),
            ];

            // 创建每个链接项目
            for (index, (text, url)) in links.iter().enumerate() {
                list.spawn((
                    LinkButton {
                        url: url.to_string(),
                    },
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        margin: UiRect::bottom(Val::Px(2.0)), // 项目之间的小间隙
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(15.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                )).with_children(|button| {
                    button.spawn((
                        Text::new(format!("{}. {}", index + 1, text)),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
            }
        });
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_game_scene)
        .add_systems(Update, handle_link_clicks)
        .run();
}