use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::sprite::*;
use bevy::window::WindowMode;  // 显式导入WindowMode
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

#[derive(Component)]
struct ScalingCamera;

#[derive(Resource)]
struct BaseResolution {
    width: f32,
    height: f32,
    aspect_ratio: f32,
}


fn toggle_fullscreen(
    input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
) {
    if input.just_pressed(KeyCode::F4) {
        if let Ok(mut window) = windows.single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                _ => WindowMode::Windowed,
            };
        }
    }
}
fn main() {
    let app_window = Some(Window {
        title: "I am a window!".into(),
        // resizable: false, // 按你之前的要求禁用调整大小
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
        ..default()
    });
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: app_window,
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK)) // 设置背景清除颜色为黑色
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     primary_window: Some(Window {
        //         resolution: WindowResolution::new(1152.0, 648.0)
        //             .with_scale_factor_override(1.0),
        //         title: "固定比例视差背景聚光灯效果".to_string(),
        //         resizable: true,
        //         ..default()
        //     }),
        //     ..default()
        // }))
        .add_plugins(Material2dPlugin::<ParallaxSpotlightMaterial>::default())
        .insert_resource(BaseResolution {
                width: 1152.0,  // 恢复为1152
                    height: 648.0,
                    aspect_ratio: 1152.0 / 648.0,  // 约1.78的宽高比(16:9)
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_camera_scaling,
            update_spotlight,
            update_parallax,
            toggle_fullscreen,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParallaxSpotlightMaterial>>,
    asset_server: Res<AssetServer>,
    base_resolution: Res<BaseResolution>,
) {
    commands.spawn((
        Camera2d,
        ScalingCamera,
    ));

    // 加载背景图片
    let background_image = asset_server.load("background/bg2.png");

    // 创建聚光灯材质
    let spotlight_material = materials.add(ParallaxSpotlightMaterial {
        mouse_pos: Vec2::new(0.5, 0.5),
        spotlight_radius: 120.0,
        edge_softness: 60.0,
        brightness_factor: 6.0,
        background_texture: Some(background_image),
    });

    // 创建视差背景层 - 始终使用固定尺寸
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(base_resolution.width, base_resolution.height))),
        MeshMaterial2d(spotlight_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ParallaxLayer { depth: 1.0 },
    ));
}

// 更新相机缩放以保持固定比例
fn update_camera_scaling(
    mut camera_query: Query<&mut Transform, With<ScalingCamera>>,
    windows: Query<&Window>,
    base_resolution: Res<BaseResolution>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };

    let window_width = window.resolution.width();
    let window_height = window.resolution.height();
    let window_aspect = window_width / window_height;

    // 计算缩放比例，保持图片完整显示
    let scale = if window_aspect > base_resolution.aspect_ratio {
        // 窗口更宽，以高度为准
        window_height / base_resolution.height
    } else {
        // 窗口更高，以宽度为准
        window_width / base_resolution.width
    };

    // 应用缩放
    camera_transform.scale = Vec3::splat(1.0 / scale);
}

fn update_spotlight(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut materials: ResMut<Assets<ParallaxSpotlightMaterial>>,
    mask_query: Query<&MeshMaterial2d<ParallaxSpotlightMaterial>, With<ParallaxLayer>>,
    base_resolution: Res<BaseResolution>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for material_handle in mask_query.iter() {
                if let Some(material) = materials.get_mut(material_handle.id()) {
                    // 将世界坐标转换为UV坐标
                    let uv_x = (world_position.x + base_resolution.width * 0.5) / base_resolution.width;
                    let uv_y = 1.0 - (world_position.y + base_resolution.height * 0.5) / base_resolution.height;
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
                let parallax_strength = 0.00;
                let offset_x = world_position.x * parallax_strength * layer.depth;
                let offset_y = world_position.y * parallax_strength * layer.depth;
                
                transform.translation.x = offset_x;
                transform.translation.y = offset_y;
            }
        }
    }
}