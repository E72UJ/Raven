use bevy::prelude::*;
use bevy::ecs::system::ParamSet;

// Transition State Enum
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum TransitionState {
    None,
    FadeOut,
    FadeIn,
    Dissolve,
}

impl Default for TransitionState {
    fn default() -> Self {
        TransitionState::None
    }
}

// Transition Timer Resource
#[derive(Resource)]
struct TransitionTimer(Timer);

// Transition Overlay Component
#[derive(Component)]
struct TransitionOverlay;

// Main Sprite Component
#[derive(Component)]
struct MainSprite;

// Main Text Component
#[derive(Component)]
struct MainText;

// Background Component
#[derive(Component)]
struct Background;

// Game State Resource
#[derive(Resource)]
struct GameState {
    current_line: usize,
    texts: Vec<String>,
    colors: Vec<Color>,
    is_transitioning: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy 0.16 Transition System Example".to_string(),
                resolution: (1220.0, 680.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<TransitionState>()
        .add_systems(Startup, setup_system)
        .add_systems(Update, (
            handle_input_system,
            fade_out_system,
            fade_in_system,
            dissolve_system,
        ))
        .run();
}

fn setup_system(
    mut commands: Commands,
) {
    // Create 2D Camera
    commands.spawn(Camera2d);

    // Create Background
    commands.spawn((
        Background,
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.3),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    // Create Main Sprite
    commands.spawn((
        MainSprite,
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.2),
            custom_size: Some(Vec2::new(200.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 1.0),
    ));

    // Create Text
    commands.spawn((
        MainText,
        Text2d::new("Click Space or Left Mouse to switch content"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -200.0, 1.0),
    ));

    // Create Transition Overlay
    commands.spawn((
        TransitionOverlay,
        Sprite {
            color: Color::BLACK.with_alpha(0.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
    ));

    // Initialize Game State
    commands.insert_resource(GameState {
        current_line: 0,
        texts: vec![
            "Click Space or Left Mouse to switch content".to_string(),
            "This is the second text content".to_string(),
            "This is the third text content".to_string(),
            "This is the last text".to_string(),
        ],
        colors: vec![
            Color::srgb(0.8, 0.2, 0.2), // Red
            Color::srgb(0.2, 0.8, 0.2), // Green
            Color::srgb(0.2, 0.2, 0.8), // Blue
            Color::srgb(0.8, 0.8, 0.2), // Yellow
        ],
        is_transitioning: false,
    });

    // Initialize Transition Timer
    commands.insert_resource(TransitionTimer(Timer::from_seconds(2.5, TimerMode::Once)));
}

fn handle_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<TransitionState>>,
    current_state: Res<State<TransitionState>>,
    mut game_state: ResMut<GameState>,
) {
    let input_triggered = keys.just_pressed(KeyCode::Space) 
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left);

    if input_triggered 
        && *current_state.get() == TransitionState::None 
        && !game_state.is_transitioning
        && game_state.current_line < game_state.texts.len() - 1 
    {
        game_state.is_transitioning = true;
        next_state.set(TransitionState::Dissolve);
    }

    // ESC to exit
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn fade_out_system(
    mut overlay_query: Query<&mut Sprite, With<TransitionOverlay>>,
    time: Res<Time>,
    mut timer: ResMut<TransitionTimer>,
    mut next_state: ResMut<NextState<TransitionState>>,
    current_state: Res<State<TransitionState>>,
) {
    if *current_state.get() != TransitionState::FadeOut {
        return;
    }

    timer.0.tick(time.delta());

    for mut sprite in &mut overlay_query {
        let progress = timer.0.elapsed_secs() / timer.0.duration().as_secs_f32();
        let alpha = progress.min(1.0);
        sprite.color = Color::BLACK.with_alpha(alpha);

        if timer.0.finished() {
            timer.0.reset();
            next_state.set(TransitionState::FadeIn);
        }
    }
}

fn fade_in_system(
    mut overlay_query: Query<&mut Sprite, With<TransitionOverlay>>,
    mut background_query: Query<&mut Sprite, (With<Background>, Without<TransitionOverlay>)>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut timer: ResMut<TransitionTimer>,
    mut next_state: ResMut<NextState<TransitionState>>,
    current_state: Res<State<TransitionState>>,
) {
    if *current_state.get() != TransitionState::FadeIn {
        return;
    }

    timer.0.tick(time.delta());

    // Update content during transition
    if timer.0.elapsed_secs() / timer.0.duration().as_secs_f32() > 0.5 && game_state.current_line < game_state.texts.len() - 1 {
        game_state.current_line += 1;

        // Update background color (example)
        for mut background in &mut background_query {
            background.color = game_state.colors[game_state.current_line % game_state.colors.len()];
        }
    }

    // Handle fade-in effect
    for mut sprite in &mut overlay_query {
        let progress = timer.0.elapsed_secs() / timer.0.duration().as_secs_f32();
        let alpha = 1.0 - progress.min(1.0);
        sprite.color = Color::BLACK.with_alpha(alpha);

        if timer.0.finished() {
            sprite.color = Color::BLACK.with_alpha(0.0);
            timer.0.reset();
            game_state.is_transitioning = false;
            next_state.set(TransitionState::None);
        }
    }
}

fn dissolve_system(
    mut sprite_queries: ParamSet<(
        Query<&mut Sprite, With<TransitionOverlay>>,
        Query<&mut Sprite, With<MainSprite>>,
    )>,
    mut main_text_query: Query<&mut TextColor, With<MainText>>,
    mut text2d_query: Query<&mut Text2d, With<MainText>>,
    time: Res<Time>,
    mut timer: ResMut<TransitionTimer>,
    mut next_state: ResMut<NextState<TransitionState>>,
    current_state: Res<State<TransitionState>>,
    mut game_state: ResMut<GameState>,
) {
    if *current_state.get() != TransitionState::Dissolve {
        return;
    }

    timer.0.tick(time.delta());
    let progress = timer.0.elapsed_secs() / timer.0.duration().as_secs_f32();

    // Dissolve effect for main sprite
    for mut sprite in sprite_queries.p1().iter_mut() {
        sprite.color = game_state.colors[game_state.current_line % game_state.colors.len()].with_alpha(1.0 - progress.min(1.0));
    }

    // Dissolve effect for main text
    for mut text_color in &mut main_text_query {
        text_color.0 = Color::WHITE.with_alpha(1.0 - progress.min(1.0));
    }

    // Update text content during transition
    if progress > 0.5 && game_state.current_line < game_state.texts.len() - 1 {
        game_state.current_line += 1;

        // Update text content
        for mut text in &mut text2d_query {
            text.0 = game_state.texts[game_state.current_line].clone();
        }

        // Restore text visibility for fade-in
        for mut text_color in &mut main_text_query {
            text_color.0 = Color::WHITE.with_alpha(progress - 0.5);
        }

        // Restore sprite visibility for fade-in
        for mut sprite in sprite_queries.p1().iter_mut() {
            sprite.color = game_state.colors[game_state.current_line % game_state.colors.len()].with_alpha(progress - 0.5);
        }
    }

    // Dissolve effect for transition overlay
    for mut sprite in sprite_queries.p0().iter_mut() {
        sprite.color = Color::BLACK.with_alpha(progress.min(1.0) * 0.3); // 轻微的覆盖效果
    }

    if timer.0.finished() {
        // Reset overlay to transparent
        for mut sprite in sprite_queries.p0().iter_mut() {
            sprite.color = Color::BLACK.with_alpha(0.0);
        }

        // Ensure full visibility
        for mut text_color in &mut main_text_query {
            text_color.0 = Color::WHITE;
        }

        for mut sprite in sprite_queries.p1().iter_mut() {
            sprite.color = game_state.colors[game_state.current_line % game_state.colors.len()];
        }

        timer.0.reset();
        game_state.is_transitioning = false;
        next_state.set(TransitionState::None);
    }
}