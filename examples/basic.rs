
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, hello_world_system)
        .run();
}

fn setup() {
    println!("启动设置");
}

fn hello_world_system() {
    println!("Hello, world!");
}