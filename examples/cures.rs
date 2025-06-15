use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::sprite::*;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct ParallaxSpotlightMaterial {
    #[uniform(0)]
    mouse_pos: Vec2,
    #[uniform(1)]
    spotlight_radius: f32,
    #[uniform(2)]
    edge_softness: f32,
    #[uniform(3)]
    brightness_factor: f32,
    #[texture(4)]
    #[sampler(5)]
    background_texture: Option<Handle<Image>>,
}

impl Material2d for ParallaxSpotlightMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/parallax_spotlight.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
struct ParallaxLayer {
    depth: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
               resolution: WindowResolution::new(1152.0, 648.0)
                    .with_scale_factor_override(1.0), // 基础缩放
                title: "视差背景聚光灯效果".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(Material2dPlugin::<ParallaxSpotlightMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (update_spotlight, update_parallax))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParallaxSpotlightMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);

    // 加载背景图片
    let background_image = asset_server.load("background/bg2.png");

    // 创建聚光灯材质
    let spotlight_material = materials.add(ParallaxSpotlightMaterial {
        mouse_pos: Vec2::new(0.5, 0.5),
        spotlight_radius: 120.0,
        edge_softness: 60.0,
        brightness_factor: 3.0, // 亮区的亮度因子，1.0是原亮度，大于1.0会变亮
        background_texture: Some(background_image),
    });

    // 创建视差背景层
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1152.0, 648.0))),
        MeshMaterial2d(spotlight_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ParallaxLayer { depth: 1.0 },
    ));
}

fn update_spotlight(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut materials: ResMut<Assets<ParallaxSpotlightMaterial>>,
    mask_query: Query<&MeshMaterial2d<ParallaxSpotlightMaterial>, With<ParallaxLayer>>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for material_handle in mask_query.iter() {
                if let Some(material) = materials.get_mut(material_handle.id()) {
                    // 将世界坐标转换为UV坐标 (0.0 到 1.0)
                    let uv_x = (world_position.x + 576.0) / 1152.0;
                    let uv_y = 1.0 - (world_position.y + 324.0) / 648.0;
                    material.mouse_pos = Vec2::new(uv_x.clamp(0.0, 1.0), uv_y.clamp(0.0, 1.0));
                }
            }
        }
    }
}

fn update_parallax(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut parallax_query: Query<(&mut Transform, &ParallaxLayer), (With<ParallaxLayer>, Without<Camera>)>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for (mut transform, layer) in parallax_query.iter_mut() {
                // 计算视差偏移
                let parallax_strength = 0.02; // 调整这个值来控制视差强度
                let offset_x = world_position.x * parallax_strength * layer.depth;
                let offset_y = world_position.y * parallax_strength * layer.depth;
                
                transform.translation.x = offset_x;
                transform.translation.y = offset_y;
            }
        }
    }
}