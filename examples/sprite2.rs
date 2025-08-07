use bevy::prelude::*;
use bevy::sprite::Anchor;  
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    
    // 创建精灵
    let mut sprite = Sprite::from_image(asset_server.load("background/one.png"));
    sprite.anchor = Anchor::TopLeft;        // 居中（默认）
    // 使用新的 SpriteImageMode 变体
    // 如果想要拉伸效果，可以使用：
    // sprite.image_mode = SpriteImageMode::Scale(ScalingMode::Stretch);
    
    // 或者其他选项：
    // sprite.image_mode = SpriteImageMode::Auto;  // 自动模式
    // sprite.image_mode = SpriteImageMode::Tiled { 
    //     tile_x: true, 
    //     tile_y: true, 
    //     stretch_value: 1.0 
    // };  // 平铺模式
    
    // 设置自定义尺寸
    sprite.custom_size = Some(Vec2::new(400.0, 400.0));
    
    commands.spawn((
        sprite,
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));
}
