use bevy::ecs::system::command;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BorderMaterial {
    #[uniform(0)]
    border_width: f32,
    #[uniform(0)]
    border_color: LinearRgba,
    #[uniform(0)]
    inner_color_multiplier: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    base_texture: Handle<Image>,
}

impl Material2d for BorderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/border.wgsl".into()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<BorderMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_border)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<BorderMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // 生成摄像机
    commands.spawn(Camera2d);

    // 创建边框材质
    let border_material = materials.add(BorderMaterial {
        border_width: 0.05,
        border_color: LinearRgba::RED,
        inner_color_multiplier: LinearRgba::WHITE,
        base_texture: asset_server.load("characters/protagonist/default.png"),
    });

    // 生成带边框的精灵
    commands.spawn((
        Name::new("bordered_sprite"),
        Mesh2d(meshes.add(Rectangle::new(400.0, 600.0))),
        MeshMaterial2d(border_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
       commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(50.0),
                right: Val::Px(50.0),
                height: Val::Px(170.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),

        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("textbox"),
                Text::new("hello hello hello"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn update_border(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<BorderMaterial>>,
    query: Query<&MeshMaterial2d<BorderMaterial>>,
) {
    for material_handle in query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            // 上箭头增加边框宽度
            if keyboard_input.just_pressed(KeyCode::ArrowUp) {
                material.border_width = (material.border_width + 0.01).min(0.2);
                println!("边框宽度: {:.2}", material.border_width);
            }
            // 下箭头减少边框宽度
            if keyboard_input.just_pressed(KeyCode::ArrowDown) {
                material.border_width = (material.border_width - 0.01).max(0.0);
                println!("边框宽度: {:.2}", material.border_width);
            }
            // 空格键切换边框颜色
            if keyboard_input.just_pressed(KeyCode::Space) {
                material.border_color = if material.border_color.red > 0.5 {
                    LinearRgba::BLUE
                } else {
                    LinearRgba::RED
                };
                println!("边框颜色已切换");
            }
            // R键切换内部颜色效果
            if keyboard_input.just_pressed(KeyCode::KeyR) {
                material.inner_color_multiplier = if material.inner_color_multiplier.red > 0.9 {
                    LinearRgba::new(0.5, 0.5, 0.5, 1.0) // 灰色效果
                } else {
                    LinearRgba::WHITE // 正常颜色
                };
                println!("内部颜色效果已切换");
            }
        }
    }
}