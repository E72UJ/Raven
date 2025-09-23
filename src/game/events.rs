
use bevy::prelude::*;

/// 切换游戏菜单的事件
#[derive(Event)]
pub struct ToggleGameMenuEvent;

/// 关闭游戏菜单的事件
#[derive(Event)]
pub struct CloseGameMenuEvent;

/// 打开游戏菜单的事件
#[derive(Event)]
pub struct OpenGameMenuEvent;

/// 菜单状态变化事件
#[derive(Event)]
pub struct MenuStateChangedEvent {
    pub is_open: bool,
}


#[derive(Event)]
pub struct ToggleMenuEvent;