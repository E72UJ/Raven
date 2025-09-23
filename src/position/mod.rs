use bevy::prelude::*;

pub struct PositionPlugin;

impl Plugin for PositionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, startup_system)
            .add_systems(Update, update_system);
    }
}

fn startup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("定位插件启动");
}

fn update_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<PositionComponent>>,
) {
    // 更新逻辑
}

#[derive(Component)]
pub struct PositionComponent;