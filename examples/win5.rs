use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<VolumeControl>()
        .init_resource::<SliderAssets>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            slider_interaction, 
            update_slider_display, 
            volume_control_system, 
            button_interaction,
            slider_hover_system
        ))
        .run();
}

#[derive(Resource)]
struct VolumeControl {
    volume: f32, // 0.0 到 1.0
}

impl Default for VolumeControl {
    fn default() -> Self {
        Self { volume: 0.6 }
    }
}

#[derive(Resource)]
struct SliderAssets {
    // 水平滚动条资源
    horizontal_idle_bar: Handle<Image>,
    horizontal_hover_bar: Handle<Image>,
    horizontal_idle_thumb: Handle<Image>,
    horizontal_hover_thumb: Handle<Image>,
    
    // 垂直滚动条资源
    vertical_idle_bar: Handle<Image>,
    vertical_hover_bar: Handle<Image>,
    vertical_idle_thumb: Handle<Image>,
    vertical_hover_thumb: Handle<Image>,
    
    // 字体
    font: Handle<Font>,
}

impl FromWorld for SliderAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            // 加载水平滚动条图片
            horizontal_idle_bar: asset_server.load("gui/scrollbar/horizontal_idle_bar.png"),
            horizontal_hover_bar: asset_server.load("gui/scrollbar/horizontal_hover_bar.png"),
            horizontal_idle_thumb: asset_server.load("gui/scrollbar/horizontal_idle_thumb.png"),
            horizontal_hover_thumb: asset_server.load("gui/scrollbar/horizontal_hover_thumb.png"),
            
            // 加载垂直滚动条图片
            vertical_idle_bar: asset_server.load("gui/scrollbar/vertical_idle_bar.png"),
            vertical_hover_bar: asset_server.load("gui/scrollbar/vertical_hover_bar.png"),
            vertical_idle_thumb: asset_server.load("gui/scrollbar/vertical_idle_thumb.png"),
            vertical_hover_thumb: asset_server.load("gui/scrollbar/vertical_hover_thumb.png"),
            
            // 加载字体
            font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
        }
    }
}

#[derive(Component)]
struct Slider {
    value: f32,  // 0.0 到 1.0
    bar_width: f32,  // 滑块轨道的实际宽度
    bar_height: f32, // 滑块轨道的实际高度
    is_dragging: bool,
    is_horizontal: bool,
}

#[derive(Component)]
struct SliderHandle {
    is_hovering: bool,
    is_horizontal: bool,
}

#[derive(Component)]
struct SliderBar {
    is_hovering: bool,
    is_horizontal: bool,
}

#[derive(Component)]
struct VolumeText;

#[derive(Component)]
struct MuteButton;

#[derive(Component)]
struct MaxVolumeButton;

fn setup(mut commands: Commands, slider_assets: Res<SliderAssets>) {
    // 摄像头
    commands.spawn(Camera2d);

    // UI 根节点
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
        ))
        .with_children(|parent| {
            // 标题文字
            parent.spawn((
                Text::new("🔊 RenPy风格音量控制器"),
                TextFont {
                    font: slider_assets.font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // 水平滑块容器
            parent
                .spawn(Node {
                    width: Val::Px(400.0),
                    height: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // 水平滑块背景轨道
                    parent.spawn((
                        Node {
                            width: Val::Px(350.0),
                            height: Val::Px(20.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        ImageNode::new(slider_assets.horizontal_idle_bar.clone()),
                        SliderBar { 
                            is_hovering: false,
                            is_horizontal: true,
                        },
                        Interaction::default(),
                    ));

                    // 水平滑块手柄
                    parent.spawn((
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(190.0), // (350-24) * 0.6 = 195.6
                            top: Val::Px(30.0),
                            ..default()
                        },
                        ImageNode::new(slider_assets.horizontal_idle_thumb.clone()),
                        Slider {
                            value: 0.6,
                            bar_width: 350.0,
                            bar_height: 20.0,
                            is_dragging: false,
                            is_horizontal: true,
                        },
                        SliderHandle { 
                            is_hovering: false,
                            is_horizontal: true,
                        },
                        Button,
                    ));
                });

            // 垂直滑块容器
            parent
                .spawn(Node {
                    width: Val::Px(80.0),
                    height: Val::Px(220.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // 垂直滑块背景轨道
                    parent.spawn((
                        Node {
                            width: Val::Px(20.0),
                            height: Val::Px(200.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        ImageNode::new(slider_assets.vertical_idle_bar.clone()),
                        SliderBar { 
                            is_hovering: false,
                            is_horizontal: false,
                        },
                        Interaction::default(),
                    ));

                    // 垂直滑块手柄
                    parent.spawn((
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(-2.0),
                            bottom: Val::Px(105.6), // (200-24) * 0.6 = 105.6
                            ..default()
                        },
                        ImageNode::new(slider_assets.vertical_idle_thumb.clone()),
                        Slider {
                            value: 0.6,
                            bar_width: 20.0,
                            bar_height: 200.0,
                            is_dragging: false,
                            is_horizontal: false,
                        },
                        SliderHandle { 
                            is_hovering: false,
                            is_horizontal: false,
                        },
                        Button,
                    ));
                });

            // 音量数值显示
            parent
                .spawn((
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        padding: UiRect::all(Val::Px(15.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)),
                    BorderRadius::all(Val::Px(10.0)),
                    BorderColor(Color::srgb(0.4, 0.4, 0.5)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("音量: 60%"),
                        TextFont {
                            font: slider_assets.font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        VolumeText,
                    ));
                });

            // 音量控制按钮组
            parent
                .spawn(Node {
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    // 静音按钮
                    parent.spawn((
                        Button,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                        BorderRadius::all(Val::Px(8.0)),
                        BorderColor(Color::srgb(0.8, 0.3, 0.3)),
                        MuteButton,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("静音"),
                            TextFont {
                                font: slider_assets.font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // 最大音量按钮
                    parent.spawn((
                        Button,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                        BorderRadius::all(Val::Px(8.0)),
                        BorderColor(Color::srgb(0.3, 0.8, 0.3)),
                        MaxVolumeButton,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("最大"),
                            TextFont {
                                font: slider_assets.font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        });
}

// 悬停状态系统
fn slider_hover_system(
    mut handle_query: Query<(&mut SliderHandle, &mut ImageNode, &Interaction), (With<Button>, Without<SliderBar>)>,
    mut bar_query: Query<(&mut SliderBar, &mut ImageNode, &Interaction), (Without<Button>, Without<SliderHandle>)>,
    slider_assets: Res<SliderAssets>,
) {
    // 处理滑块手柄悬停效果
    for (mut handle, mut image, interaction) in handle_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                if !handle.is_hovering {
                    handle.is_hovering = true;
                    if handle.is_horizontal {
                        image.image = slider_assets.horizontal_hover_thumb.clone();
                    } else {
                        image.image = slider_assets.vertical_hover_thumb.clone();
                    }
                }
            }
            _ => {
                if handle.is_hovering {
                    handle.is_hovering = false;
                    if handle.is_horizontal {
                        image.image = slider_assets.horizontal_idle_thumb.clone();
                    } else {
                        image.image = slider_assets.vertical_idle_thumb.clone();
                    }
                }
            }
        }
    }

    // 处理滑块轨道悬停效果
    for (mut bar, mut image, interaction) in bar_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                if !bar.is_hovering {
                    bar.is_hovering = true;
                    if bar.is_horizontal {
                        image.image = slider_assets.horizontal_hover_bar.clone();
                    } else {
                        image.image = slider_assets.vertical_hover_bar.clone();
                    }
                }
            }
            _ => {
                if bar.is_hovering {
                    bar.is_hovering = false;
                    if bar.is_horizontal {
                        image.image = slider_assets.horizontal_idle_bar.clone();
                    } else {
                        image.image = slider_assets.vertical_idle_bar.clone();
                    }
                }
            }
        }
    }
}

fn slider_interaction(
    mut slider_query: Query<(&mut Slider, &mut Node, &Interaction), With<SliderHandle>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
    mut volume_control: ResMut<VolumeControl>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    
    let Ok(_window) = windows.single() else {
        return;
    };

    for (mut slider, mut style, interaction) in slider_query.iter_mut() {
        // 检查是否开始拖拽
        if *interaction == Interaction::Pressed && mouse_input.just_pressed(MouseButton::Left) {
            slider.is_dragging = true;
        }
        
        // 检查是否停止拖拽
        if mouse_input.just_released(MouseButton::Left) {
            slider.is_dragging = false;
        }

        // 如果正在拖拽，处理鼠标移动
        if slider.is_dragging {
            for cursor_event in cursor_moved_events.read() {
                if let Ok(world_position) = camera.viewport_to_world_2d(
                    camera_transform,
                    cursor_event.position
                ) {
                    if slider.is_horizontal {
                        // 水平滑块逻辑 - 修正坐标计算
                        let container_center_x = 0.0; // 容器中心X坐标
                        let bar_left = container_center_x - slider.bar_width / 2.0;
                        let thumb_width = 24.0;
                        let slider_relative_x = world_position.x - bar_left;
                        let max_x = slider.bar_width - thumb_width;
                        let clamped_x = slider_relative_x.clamp(0.0, max_x);
                        
                        slider.value = clamped_x / max_x;
                        style.left = Val::Px(clamped_x);
                    } else {
                        // 垂直滑块逻辑 - 修正坐标计算
                        let container_center_y = 0.0; // 容器中心Y坐标
                        let bar_bottom = container_center_y - slider.bar_height / 2.0;
                        let thumb_height = 24.0;
                        let slider_relative_y = world_position.y - bar_bottom;
                        let max_y = slider.bar_height - thumb_height;
                        let clamped_y = slider_relative_y.clamp(0.0, max_y);
                        
                        slider.value = clamped_y / max_y;
                        style.bottom = Val::Px(clamped_y);
                    }
                    
                    // 更新全局音量控制
                    volume_control.volume = slider.value;
                }
            }
        }
    }
}

fn button_interaction(
    mut volume_control: ResMut<VolumeControl>,
    mut slider_query: Query<(&mut Slider, &mut Node), With<SliderHandle>>,
    mute_query: Query<&Interaction, (With<MuteButton>, Changed<Interaction>)>,
    max_query: Query<&Interaction, (With<MaxVolumeButton>, Changed<Interaction>)>,
) {
    // 处理静音按钮
    for interaction in mute_query.iter() {
        if *interaction == Interaction::Pressed {
            volume_control.volume = 0.0;
            for (mut slider, mut style) in slider_query.iter_mut() {
                slider.value = 0.0;
                if slider.is_horizontal {
                    style.left = Val::Px(0.0);
                } else {
                    style.bottom = Val::Px(0.0);
                }
            }
        }
    }

    // 处理最大音量按钮
    for interaction in max_query.iter() {
        if *interaction == Interaction::Pressed {
            volume_control.volume = 1.0;
            for (mut slider, mut style) in slider_query.iter_mut() {
                slider.value = 1.0;
                let thumb_size = 24.0;
                if slider.is_horizontal {
                    let max_pos = slider.bar_width - thumb_size;
                    style.left = Val::Px(max_pos);
                } else {
                    let max_pos = slider.bar_height - thumb_size;
                    style.bottom = Val::Px(max_pos);
                }
            }
        }
    }
}

fn update_slider_display(
    slider_query: Query<&Slider, (With<SliderHandle>, Changed<Slider>)>,
    mut text_query: Query<&mut Text, With<VolumeText>>,
) {
    for slider in slider_query.iter() {
        let volume_percent = (slider.value * 100.0) as i32;
        
        // 更新音量文字显示
        for mut text in text_query.iter_mut() {
            **text = format!("音量: {}%", volume_percent);
        }
    }
}

// 音量控制系统
fn volume_control_system(
    volume_control: Res<VolumeControl>,
    time: Res<Time>,
) {
    static mut TIMER: f32 = 0.0;
    unsafe {
        TIMER += time.delta_secs();
        if TIMER >= 2.0 {
            println!("🎵 当前音量: {:.1}%", volume_control.volume * 100.0);
            TIMER = 0.0;
        }
    }
}

// 辅助函数
pub fn get_current_volume(volume_control: &VolumeControl) -> f32 {
    volume_control.volume
}

pub fn set_volume(volume_control: &mut VolumeControl, new_volume: f32) {
    volume_control.volume = new_volume.clamp(0.0, 1.0);
}
