use bevy::{prelude::*, sprite::Anchor};
use std::fmt::Debug;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup, setup_atlas))
        .add_systems(Update, (move_sprite, animate_sprite))
        .run();
}

fn move_sprite(
    time: Res<Time>,
    mut sprite: Query<&mut Transform, (Without<Sprite>, With<Children>)>,
) {
    let t = time.elapsed_secs() * 0.1;
    for mut transform in &mut sprite {
        let new = Vec2 {
            x: 50.0 * ops::sin(t),
            y: 50.0 * ops::sin(t * 2.0),
        };
        transform.translation.x = new.x;
        transform.translation.y = new.y;
    }
}

/// Set up a scene that tests all sprite anchor types.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let len = 1200.0;
    let sprite_size = Vec2::splat(len / 2.0);

    // 只创建一个 sprite 实体
    commands
    .spawn((
        Sprite {
            image: asset_server.load("fps/6.png"),
            // custom_size: Some(Vec2::new.0, 543.0)), // 设定精灵大小
            // custom_size: Some(sprite_size),
            // color: Color::srgb(1.0, 0.0, 0.0),
            // anchor: Anchor::Center,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Pickable::default(),
    )
)
// .observe(recolor_on::<Pointer<Over>>(Color::srgb(0.0, 1.0, 1.0)))
.observe(on_hover_enter)
// .observe(recolor_on::<Pointer<Out>>(Color::BLACK))
.observe(recolor_on::<Pointer<Pressed>>(Color::srgb(1.0, 1.0, 0.0)))
.observe(recolor_on::<Pointer<Released>>(Color::srgb(0.0, 1.0, 1.0)));
}


#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        let Some(texture_atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        timer.tick(time.delta());

        if timer.just_finished() {
            texture_atlas.index = if texture_atlas.index == indices.last {
                indices.first
            } else {
                texture_atlas.index + 1
            };
        }
    }
}

fn setup_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(24, 24), 7, 1, None, None);
    let texture_atlas_layout_handle = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };
    commands
        .spawn((
            Sprite::from_atlas_image(
                texture_handle,
                TextureAtlas {
                    layout: texture_atlas_layout_handle,
                    index: animation_indices.first,
                },
            ),
            Transform::from_xyz(300.0, 0.0, 0.0).with_scale(Vec3::splat(6.0)),
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Pickable::default(),
        ))
        .observe(recolor_on::<Pointer<Over>>(Color::srgb(0.0, 1.0, 1.0)))
        .observe(recolor_on::<Pointer<Out>>(Color::srgb(1.0, 1.0, 1.0)))
        .observe(recolor_on::<Pointer<Pressed>>(Color::srgb(1.0, 1.0, 0.0)))
        .observe(recolor_on::<Pointer<Released>>(Color::srgb(0.0, 1.0, 1.0)));
}

// An observer listener that changes the target entity's color.
fn recolor_on<E: Debug + Clone + Reflect>(color: Color) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.target()) else {
            return;
        };
        sprite.color = color;
    }
}

fn on_hover_enter(
    trigger: Trigger<Pointer<Over>>, 
    mut sprites: Query<&mut Sprite>,
) {
    if let Ok(mut sprite) = sprites.get_mut(trigger.target) { // 使用 trigger.target
        sprite.color = Color::srgb(0.0, 1.0, 1.0);
        
        println!("鼠标悬停进入！");
        
    }
}