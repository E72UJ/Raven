use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::sprite::*;
use bevy::window::WindowResolution;
use bevy::render::mesh::MeshAabb; // 添加这个导入

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
}

#[derive(Resource)]
struct CurrentBackgroundSize {
    width: f32,
    height: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1152.0, 648.0)
                    .with_scale_factor_override(1.0),
                title: "自适应视差背景聚光灯效果".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(Material2dPlugin::<ParallaxSpotlightMaterial>::default())
        .insert_resource(BaseResolution {
            width: 1152.0,
            height: 648.0,
        })
        .insert_resource(CurrentBackgroundSize {
            width: 1152.0,
            height: 648.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_camera_and_background,
            update_spotlight,
            update_parallax,
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
    let background_image = asset_server.load("gui/game3.png");

    // 创建聚光灯材质
    let spotlight_material = materials.add(ParallaxSpotlightMaterial {
        mouse_pos: Vec2::new(0.5, 0.5),
        spotlight_radius: 120.0,
        edge_softness: 60.0,
        brightness_factor: 3.0,
        background_texture: Some(background_image),
    });

    // 创建视差背景层
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(base_resolution.width, base_resolution.height))),
        MeshMaterial2d(spotlight_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ParallaxLayer { depth: 1.0 },
    ));
}

// 更新相机和背景以填充整个窗口，避免黑边
fn update_camera_and_background(
    mut camera_query: Query<&mut Transform, With<ScalingCamera>>,
    windows: Query<&Window>,
    base_resolution: Res<BaseResolution>,
    mut current_size: ResMut<CurrentBackgroundSize>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_handles: Query<&Mesh2d, With<ParallaxLayer>>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };

    let window_width = window.resolution.width();
    let window_height = window.resolution.height();
    let window_aspect = window_width / window_height;
    let base_aspect = base_resolution.width / base_resolution.height;

    let (new_width, new_height, scale) = if window_aspect > base_aspect {
        // 窗口比图片更宽，需要扩展高度来填充
        let scale = window_width / base_resolution.width;
        let new_height = window_height / scale;
        (base_resolution.width, new_height, scale)
    } else {
        // 窗口比图片更高，需要扩展宽度来填充
        let scale = window_height / base_resolution.height;
        let new_width = window_width / scale;
        (new_width, base_resolution.height, scale)
    };

    // 调整相机缩放
    camera_transform.scale = Vec3::splat(1.0 / scale);
    
    // 更新当前背景尺寸资源
    current_size.width = new_width;
    current_size.height = new_height;
    
    // 更新背景网格大小以匹配新的视野
    for mesh_handle in mesh_handles.iter() {
        if let Some(mesh) = meshes.get_mut(mesh_handle.id()) {
            *mesh = Rectangle::new(new_width, new_height).into();
        }
    }
}

fn update_spotlight(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut materials: ResMut<Assets<ParallaxSpotlightMaterial>>,
    mask_query: Query<&MeshMaterial2d<ParallaxSpotlightMaterial>, With<ParallaxLayer>>,
    current_size: Res<CurrentBackgroundSize>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for material_handle in mask_query.iter() {
                if let Some(material) = materials.get_mut(material_handle.id()) {
                    // 将世界坐标转换为UV坐标，基于当前背景尺寸
                    let uv_x = (world_position.x + current_size.width * 0.5) / current_size.width;
                    let uv_y = 1.0 - (world_position.y + current_size.height * 0.5) / current_size.height;
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
                let parallax_strength = 0.01;
                let offset_x = world_position.x * parallax_strength * layer.depth;
                let offset_y = world_position.y * parallax_strength * layer.depth;
                
                transform.translation.x = offset_x;
                transform.translation.y = offset_y;
            }
        }
    }
}