// src/audio/mod.rs
use bevy::prelude::*;
use bevy::audio::Volume;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        // 如果需要音频系统的话可以在这里添加系统
    }
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
pub fn play_audio(commands: &mut Commands, asset_server: &Res<AssetServer>, audio_path: &str) -> Entity {
    let audio_handle = asset_server.load(audio_path);
    commands.spawn((
        AudioPlayer::new(audio_handle),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(1.0)),
    )).id() // 添加.id()来返回Entity
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
    audio_path: &str, 
    volume: f32
) {
    let audio_handle = asset_server.load(audio_path);
    commands.spawn((
        AudioPlayer::new(audio_handle),
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
    audio_path: &str, 
    volume: f32
) {
    let audio_handle = asset_server.load(audio_path);
    commands.spawn((
        AudioPlayer::new(audio_handle),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
    ));
}