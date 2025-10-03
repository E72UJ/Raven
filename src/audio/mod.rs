// src/audio/mod.rs
use bevy::audio::Volume;
use bevy::prelude::*;
use std::collections::HashMap;
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioManager>();
    }
}
#[derive(Resource, Default)]
pub struct AudioManager {
    pub playing_audio: HashMap<String, Entity>,
}

/// 播放音频文件的简单函数
///
/// # 参数
/// * `commands` - 用于生成音频实体的Commands
/// * `asset_server` - 用于加载音频资源的AssetServer
/// * `audio_path` - 音频文件的相对路径（相对于assets目录）
///
/// # 示例
/// ```
/// use crate::audio::play_audio;
///
/// fn some_system(mut commands: Commands, asset_server: Res<AssetServer>) {
///     play_audio(&mut commands, &asset_server, "audio/button_click.ogg");
/// }
/// ```
pub fn play_audio(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    audio_path: &'static str,
) -> Entity {
    commands
        .spawn((
            AudioPlayer::new(asset_server.load(audio_path)),
            PlaybackSettings::ONCE.with_volume(Volume::Linear(1.0)),
        ))
        .id() // 添加.id()来返回Entity
}

/// 播放音频文件并设置音量
///
/// # 参数
/// * `commands` - 用于生成音频实体的Commands
/// * `asset_server` - 用于加载音频资源的AssetServer
/// * `audio_path` - 音频文件的相对路径
/// * `volume` - 音量大小（0.0 到 1.0）
pub fn play_audio_with_volume(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    audio_path: &'static str,
    volume: f32,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(audio_path)),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(volume)),
    ));
}

/// 循环播放音频文件
///
/// # 参数
/// * `commands` - 用于生成音频实体的Commands
/// * `asset_server` - 用于加载音频资源的AssetServer
/// * `audio_path` - 音频文件的相对路径
/// * `volume` - 音量大小（0.0 到 1.0）
pub fn play_audio_loop(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    audio_path: &'static str,
    volume: f32,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(audio_path)),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
    ));
}

pub fn stop_all_audio(commands: &mut Commands, audio_manager: &mut ResMut<AudioManager>) {
    for (_tag, entity) in audio_manager.playing_audio.drain() {
        // 直接尝试销毁，即使实体不存在也不会出错
        commands.entity(entity).despawn();
    }
}
pub fn stop_all_audio_system(mut commands: Commands, mut audio_manager: ResMut<AudioManager>) {
    for (_tag, entity) in audio_manager.playing_audio.drain() {
        commands.entity(entity).despawn();
    }
}
