use bevy::prelude::*;
use crate::GameScene;  // 导入 GameScene

// 添加事件定义
#[derive(Event)]
pub struct ToggleMenuEvent;

#[derive(Event)]
pub struct RollbackEvent;  // 新增回退事件

#[derive(Event)]
pub struct ToggleAutoPlayEvent;  // 新增自动播放事件

#[derive(Component)]
pub struct ToolbarContainer;



#[derive(Component)]
pub enum ToolbarButton {
    Rollback,
    History,
    Skip,
    Auto,
    Save,
    Load,
    Settings,
}

pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ToggleMenuEvent>()  // 添加事件
            .add_event::<RollbackEvent>()  // 注册回退事件
            .add_event::<ToggleAutoPlayEvent>()  // 注册自动播放事件
            .add_systems(OnEnter(GameScene::Game), setup_toolbar)  // 只在进入游戏状态时创建
            .add_systems(OnExit(GameScene::Game), cleanup_toolbar)  // 离开游戏状态时清理
            .add_systems(Update, (
                handle_toolbar_buttons,
                // toolbar_system,  // 添加新系统
            ).run_if(in_state(GameScene::Game)));  // 只在游戏状态下运行
    }
}

fn setup_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font_handle = asset_server.load("fonts/SarasaFixedHC-Light.ttf");
    
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),  // 稍微调小高度
                position_type: PositionType::Absolute,
                bottom: Val::Px(1.0),   // 确保在最底部
                left: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,  // 对齐到底部
                padding: UiRect::all(Val::Px(0.0)),
                margin: UiRect::all(Val::Px(0.0)),
                border: UiRect::all(Val::Px(0.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),  // 移除背景颜色，完全透明
            ToolbarContainer,
        ))
        .with_children(|parent| {
            let button_style = Node {
                width: Val::Px(70.0),   // 稍微调小按钮宽度
                height: Val::Px(35.0),  // 稍微调小按钮高度
                margin: UiRect::horizontal(Val::Px(3.0)),  // 减小按钮间距
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(0.0)),  // 完全移除padding
                border: UiRect::all(Val::Px(0.0)),
                ..default()
            };

            // 回退按钮
            parent.spawn((
                Button,
                button_style.clone(),
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),  // 稍微透明
                BorderRadius::all(Val::Px(3.0)),  // 稍小的圆角
                ToolbarButton::Rollback,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("回退"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,  // 稍小的字体
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 历史按钮
            parent.spawn((
                Button,
                button_style.clone(),
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                BorderRadius::all(Val::Px(3.0)),
                ToolbarButton::History,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("历史"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 快进按钮
            parent.spawn((
                Button,
                button_style.clone(),
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                BorderRadius::all(Val::Px(3.0)),
                ToolbarButton::Skip,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("快进"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 自动按钮
            parent.spawn((
                Button,
                button_style.clone(),
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                BorderRadius::all(Val::Px(3.0)),
                ToolbarButton::Auto,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("自动"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 存档按钮
            parent.spawn((
                Button,
                button_style.clone(),
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                BorderRadius::all(Val::Px(3.0)),
                ToolbarButton::Save,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("存档"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 读档按钮
            parent.spawn((
                Button,
                button_style.clone(),
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                BorderRadius::all(Val::Px(3.0)),
                ToolbarButton::Load,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("读档"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 设置按钮
            parent.spawn((
                Button,
                button_style,
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                BorderRadius::all(Val::Px(3.0)),
                ToolbarButton::Settings,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("设置"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn cleanup_toolbar(
    mut commands: Commands,
    toolbar_query: Query<Entity, With<ToolbarContainer>>,
) {
    for entity in &toolbar_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_toolbar_buttons(
    mut interaction_query: Query<
        (&Interaction, &ToolbarButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut toggle_menu_event: EventWriter<ToggleMenuEvent>,
    mut rollback_event: EventWriter<RollbackEvent>,  // 添加回退事件发送器
    mut toggle_auto_play_event: EventWriter<ToggleAutoPlayEvent>,  // 添加自动播放事件发送器
) {
    for (interaction, button_type, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.9));
                match button_type {
                    ToolbarButton::Settings => {
                        println!("设置按钮被点击");
                        toggle_menu_event.send(ToggleMenuEvent);
                    }
                    ToolbarButton::Rollback => {
                        println!("回退按钮被点击");
                        println!("回退事件被发送");
                        rollback_event.send(RollbackEvent);  // 发送回退事件
                    }
                    ToolbarButton::History => {
                        println!("历史按钮被点击");
                    }
                    ToolbarButton::Skip => {
                        println!("快进按钮被点击");
                    }
                    ToolbarButton::Auto => {
                        println!("自动按钮被点击");
                        toggle_auto_play_event.send(ToggleAutoPlayEvent);  // 发送自动播放事件
                        
                    }
                    ToolbarButton::Save => {
                        println!("存档按钮被点击");
                    }
                    ToolbarButton::Load => {
                        println!("读档按钮被点击");
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.9));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
            }
        }
    }
}


// fn toolbar_system(
//     mut toggle_menu_event: EventWriter<ToggleMenuEvent>,
//     toolbar_buttons: Query<(&Interaction, &ToolbarButton), (Changed<Interaction>, With<Button>)>,
// ) {
//     for (interaction, button_type) in toolbar_buttons.iter() {
//         if *interaction == Interaction::Pressed {
//             if matches!(button_type, ToolbarButton::Settings) {
//                 toggle_menu_event.send(ToggleMenuEvent);
//             }
//         }
//     }
// }

fn button_pressed(buttons: &Query<(&Interaction, &ToolbarButton)>, target_button: &ToolbarButton) -> bool {
    for (interaction, button_type) in buttons.iter() {
        if std::mem::discriminant(button_type) == std::mem::discriminant(target_button) 
            && *interaction == Interaction::Pressed {
            return true;
        }
    }
    false
}
