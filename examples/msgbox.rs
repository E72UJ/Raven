use bevy::prelude::*;
use bevy::window::{WindowMode, MonitorSelection};
use std::time::Duration;

// 弹出框组件
#[derive(Component)]
struct Popup {
    timer: Timer,
    fade_in_timer: Timer,
    fade_out_timer: Timer,
    is_fading_out: bool,
}

// 弹出框事件
#[derive(Event)]
struct ShowPopupEvent {
    message: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<ShowPopupEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            toggle_fullscreen,
            handle_popup_events,
            update_popups,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn toggle_fullscreen(
    input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut popup_events: EventWriter<ShowPopupEvent>,
) {
    if input.just_pressed(KeyCode::F4) {
        if let Ok(mut window) = windows.single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => {
                    popup_events.write(ShowPopupEvent {
                        message: "程序已经开启全屏(F4)".to_string(),
                    });
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                },
                _ => WindowMode::Windowed,
            };
        }
    }
}

fn handle_popup_events(
    mut commands: Commands,
    mut popup_events: EventReader<ShowPopupEvent>,
    asset_server: Res<AssetServer>, // 添加这一行
) {
    for event in popup_events.read() {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-150.0)),
                width: Val::Px(300.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)), // 初始透明
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.0)), // 初始透明
            Popup {
                timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                fade_in_timer: Timer::new(Duration::from_millis(300), TimerMode::Once),
                fade_out_timer: Timer::new(Duration::from_millis(300), TimerMode::Once),
                is_fading_out: false,
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new(&event.message),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)), // 初始透明
                TextFont {
                    font_size: 18.0,
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    ..default()
                },
            ));
        });
    }
}

fn update_popups(
    mut commands: Commands,
    mut popup_query: Query<(Entity, &mut Popup, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut text_query: Query<&mut TextColor>,
    time: Res<Time>,
) {
    for (entity, mut popup, mut bg_color, mut border_color, children) in popup_query.iter_mut() {
        popup.timer.tick(time.delta());
        
        // 淡入动画
        if !popup.fade_in_timer.finished() {
            popup.fade_in_timer.tick(time.delta());
            let progress = popup.fade_in_timer.fraction();
            let alpha = progress;
            
            bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha * 0.8);
            border_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
            
            // 更新文字透明度
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
                }
            }
        }
        
        // 开始淡出
        if popup.timer.finished() && !popup.is_fading_out {
            popup.is_fading_out = true;
            popup.fade_out_timer.reset();
        }
        
        // 淡出动画
        if popup.is_fading_out {
            popup.fade_out_timer.tick(time.delta());
            let progress = 1.0 - popup.fade_out_timer.fraction();
            let alpha = progress;
            
            bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha * 0.8);
            border_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
            
            // 更新文字透明度
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
                }
            }
            
            // 淡出完成后删除
            if popup.fade_out_timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}