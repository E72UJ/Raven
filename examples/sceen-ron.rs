use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Reflect, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MySprite {
    pub texture_path: String,
    pub size: Vec2,
}

#[derive(Component, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_type::<MySprite>()
        .register_type::<Position>()
        .register_type::<Transform>()
        .register_type::<GlobalTransform>()
        .register_type::<Visibility>()
        .register_type::<InheritedVisibility>()
        .register_type::<ViewVisibility>()
        .register_type::<Vec2>()
        .register_type::<Vec3>()
        .register_type::<Quat>()
        .add_systems(Startup, setup)
        .add_systems(Update, (load_scene_system, handle_sprite_entities))
        .run();
}

fn setup(mut commands: Commands) {
    // 在Bevy 0.16中创建2D相机
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
    
    // 在控制台打印使用说明
    println!("按空格键加载场景中的sprite实体");
}

fn load_scene_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        println!("正在加载场景...");
        let scene_handle: Handle<DynamicScene> = asset_server.load("scenes/sprite_scene.scn.ron");
        scene_spawner.spawn_dynamic(scene_handle);
    }
}

fn handle_sprite_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &MySprite, &Transform), Added<MySprite>>,
) {
    for (entity, my_sprite, transform) in query.iter() {
        println!("创建sprite实体: {}", my_sprite.texture_path);
        
        let texture = asset_server.load(&my_sprite.texture_path);
        
        // 在Bevy 0.16中使用Sprite组件
        commands.entity(entity).insert((
            Sprite {
                image: texture,
                custom_size: Some(my_sprite.size),
                ..default()
            },
        ));
    }
}