use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::sprite::*;
use bevy::window::WindowResolution;

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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameScene {
    #[default]
    Loading,
    Menu,
    Game,
    Settings,
    ParallaxBackground,
}

#[derive(Component)]
struct ParallaxLayer {
    depth: f32,
}

#[derive(Component)]
struct ScalingCamera;

#[derive(Component)]
struct SceneCleanup;

#[derive(Resource)]
struct BaseResolution {
    width: f32,
    height: f32,
    aspect_ratio: f32,
}

#[derive(Resource)]
struct GameFonts {
    title_font: Handle<Font>,
    ui_font: Handle<Font>,
    small_font: Handle<Font>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1152.0, 648.0)
                    .with_scale_factor_override(1.0),
                title: "场景切换与视差背景聚光灯效果".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(Material2dPlugin::<ParallaxSpotlightMaterial>::default())
        .init_state::<GameScene>()
        .insert_resource(BaseResolution {
            width: 1152.0,
            height: 648.0,
            aspect_ratio: 1152.0 / 648.0,
        })
        .add_systems(Startup, (setup_camera, setup_fonts))
        .add_systems(Update, (
            check_fonts_loaded.run_if(in_state(GameScene::Loading)),
            update_camera_scaling,
            update_spotlight.run_if(in_state(GameScene::ParallaxBackground)),
            update_parallax.run_if(in_state(GameScene::ParallaxBackground)),
            handle_input,
        ))
        .add_systems(OnEnter(GameScene::Loading), setup_loading_scene)
        .add_systems(OnEnter(GameScene::Menu), setup_menu_scene)
        .add_systems(OnEnter(GameScene::Game), setup_game_scene)
        .add_systems(OnEnter(GameScene::Settings), setup_settings_scene)
        .add_systems(OnEnter(GameScene::ParallaxBackground), setup_parallax_scene)
        .add_systems(OnExit(GameScene::Loading), cleanup_scene)
        .add_systems(OnExit(GameScene::Menu), cleanup_scene)
        .add_systems(OnExit(GameScene::Game), cleanup_scene)
        .add_systems(OnExit(GameScene::Settings), cleanup_scene)
        .add_systems(OnExit(GameScene::ParallaxBackground), cleanup_scene)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        ScalingCamera,
    ));
}

fn setup_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 使用默认字体，避免字体文件加载问题
    let default_font = asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf");
    
    commands.insert_resource(GameFonts {
        title_font: default_font.clone(),
        ui_font: default_font.clone(),
        small_font: default_font,
    });
}

fn setup_loading_scene(mut commands: Commands) {
    commands.spawn((
        Text::new("正在加载..."),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(300.0),
            left: Val::Px(500.0),
            ..default()
        },
        SceneCleanup,
    ));
}

fn check_fonts_loaded(
    fonts: Option<Res<GameFonts>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    if let Some(fonts) = fonts {
        // 检查字体是否加载完成
        let title_loaded = asset_server.is_loaded_with_dependencies(&fonts.title_font);
        let ui_loaded = asset_server.is_loaded_with_dependencies(&fonts.ui_font);
        let small_loaded = asset_server.is_loaded_with_dependencies(&fonts.small_font);
        
        if title_loaded && ui_loaded && small_loaded {
            next_state.set(GameScene::Menu);
        }
    }
}

fn setup_menu_scene(mut commands: Commands, fonts: Res<GameFonts>) {
    // 主标题
    commands.spawn((
        Text::new("游戏主菜单"),
        TextFont {
            font: fonts.title_font.clone(),
            font_size: 64.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));

    // 操作说明
    commands.spawn((
        Text::new("按键说明:"),
        TextFont {
            font: fonts.ui_font.clone(),
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(220.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));

    // 按键选项
    let options = [
        "1 - 进入游戏",
        "2 - 设置选项",
        "3 - 视差背景效果",
        "ESC - 退出"
    ];

    for (i, option) in options.iter().enumerate() {
        commands.spawn((
            Text::new(*option),
            TextFont {
                font: fonts.ui_font.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.9, 0.7)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(300.0 + i as f32 * 40.0),
                left: Val::Px(80.0),
                ..default()
            },
            SceneCleanup,
        ));
    }
}

fn setup_game_scene(mut commands: Commands, fonts: Res<GameFonts>) {
    // 场景标题
    commands.spawn((
        Text::new("游戏场景"),
        TextFont {
            font: fonts.title_font.clone(),
            font_size: 56.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(150.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));

    // 场景描述
    commands.spawn((
        Text::new("这里是游戏的主要内容区域\n\n在这里可以进行游戏操作和交互"),
        TextFont {
            font: fonts.ui_font.clone(),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(250.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));

    // 返回提示
    commands.spawn((
        Text::new("按 ESC 返回主菜单"),
        TextFont {
            font: fonts.small_font.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(450.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));
}

fn setup_settings_scene(mut commands: Commands, fonts: Res<GameFonts>) {
    // 场景标题
    commands.spawn((
        Text::new("设置选项"),
        TextFont {
            font: fonts.title_font.clone(),
            font_size: 56.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(150.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));

    // 设置选项列表
    let settings = [
        "音量设置: ████████░░ 80%",
        "画质设置: 高",
        "全屏模式: 开启",
        "垂直同步: 开启",
    ];

    for (i, setting) in settings.iter().enumerate() {
        commands.spawn((
            Text::new(*setting),
            TextFont {
                font: fonts.ui_font.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(280.0 + i as f32 * 45.0),
                left: Val::Px(80.0),
                ..default()
            },
            SceneCleanup,
        ));
    }

    // 返回提示
    commands.spawn((
        Text::new("按 ESC 返回主菜单"),
        TextFont {
            font: fonts.small_font.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(500.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));
}

fn setup_parallax_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParallaxSpotlightMaterial>>,
    asset_server: Res<AssetServer>,
    base_resolution: Res<BaseResolution>,
    fonts: Res<GameFonts>,
) {
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

    // 创建视差背景层
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(base_resolution.width, base_resolution.height))),
        MeshMaterial2d(spotlight_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ParallaxLayer { depth: 1.0 },
        SceneCleanup,
    ));

    // 场景标题
    commands.spawn((
        Text::new("视差背景聚光灯"),
        TextFont {
            font: fonts.title_font.clone(),
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));

    // 效果说明
    commands.spawn((
        Text::new("移动鼠标体验聚光灯效果"),
        TextFont {
            font: fonts.ui_font.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));

    // 返回提示
    commands.spawn((
        Text::new("按 ESC 返回主菜单"),
        TextFont {
            font: fonts.small_font.clone(),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(560.0),
            left: Val::Px(50.0),
            ..default()
        },
        SceneCleanup,
    ));
}

fn cleanup_scene(
    mut commands: Commands,
    query: Query<Entity, With<SceneCleanup>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameScene>>,
    mut next_state: ResMut<NextState<GameScene>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if *current_state.get() != GameScene::Loading {
            next_state.set(GameScene::Menu);
        }
    } else if *current_state.get() == GameScene::Menu {
        if keyboard_input.just_pressed(KeyCode::Digit1) {
            next_state.set(GameScene::Game);
        } else if keyboard_input.just_pressed(KeyCode::Digit2) {
            next_state.set(GameScene::Settings);
        } else if keyboard_input.just_pressed(KeyCode::Digit3) {
            next_state.set(GameScene::ParallaxBackground);
        }
    }
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
                let parallax_strength = 0.01;
                let offset_x = world_position.x * parallax_strength * layer.depth;
                let offset_y = world_position.y * parallax_strength * layer.depth;
                
                transform.translation.x = offset_x;
                transform.translation.y = offset_y;
            }
        }
    }
}