// pub mod core;
// pub mod dialogue;
// pub mod character;
// pub mod scene;
// pub mod ui;
// pub mod audio;
// pub mod input;

// use bevy::prelude::*;

// // GamePlugin 是整个游戏引擎的主入口
// pub struct GamePlugin;

// impl Plugin for GamePlugin {
//     fn build(&self, app: &mut App) {
//         app
//             // 添加核心插件
//             .add_plugins(core::CorePlugin)
//             // 添加对话系统插件
//             .add_plugins(dialogue::DialoguePlugin)
//             // 添加角色系统插件
//             .add_plugins(character::CharacterPlugin)
//             // 添加场景系统插件
//             .add_plugins(scene::ScenePlugin)
//             // 添加UI系统插件
//             .add_plugins(ui::UiPlugin)
//             // 添加音频系统插件
//             .add_plugins(audio::AudioPlugin)
//             // 添加输入系统插件
//             .add_plugins(input::InputPlugin);
//     }
// }