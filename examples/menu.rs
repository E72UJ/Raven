//! This example illustrates how to create a button that changes color and text based on its
//! interaction state, and allows switching between different scenes.

use bevy::{input_focus::InputFocus, prelude::*, winit::WinitSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        // `InputFocus` must be set for accessibility to recognize the button.
        .init_resource::<InputFocus>()
        .init_state::<GameScene>()
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .add_systems(OnEnter(GameScene::Menu), setup_menu_scene)
        .add_systems(OnEnter(GameScene::Game), setup_game_scene)
        .add_systems(OnEnter(GameScene::Settings), setup_settings_scene)
        .add_systems(OnExit(GameScene::Menu), cleanup_scene)
        .add_systems(OnExit(GameScene::Game), cleanup_scene)
        .add_systems(OnExit(GameScene::Settings), cleanup_scene)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameScene {
    #[default]
    Menu,
    Game,
    Settings,
}

#[derive(Component)]
struct SceneEntity;

#[derive(Component)]
struct MenuToGameButton;

#[derive(Component)]
struct GameToSettingsButton;

#[derive(Component)]
struct SettingsToMenuButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<GameScene>>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
            Option<&MenuToGameButton>,
            Option<&GameToSettingsButton>,
            Option<&SettingsToMenuButton>,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children, menu_to_game, game_to_settings, settings_to_menu) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *border_color = BorderColor(Color::WHITE.with_alpha(0.8)); // Glow effect on click
                *color = PRESSED_BUTTON.into();
                button.set_changed();

                // Handle scene switching based on button type
                if menu_to_game.is_some() {
                    next_state.set(GameScene::Game);
                } else if game_to_settings.is_some() {
                    next_state.set(GameScene::Settings);
                } else if settings_to_menu.is_some() {
                    next_state.set(GameScene::Menu);
                }
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                *border_color = BorderColor(Color::WHITE.with_alpha(0.6)); // Glow effect on hover
                *color = HOVERED_BUTTON.into();
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                *border_color = BorderColor(Color::WHITE); // Reset border color
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup(mut commands: Commands) {
    // ui camera
    commands.spawn(Camera2d);
}

fn setup_menu_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SceneEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        children![
            (
                Text::new("Main Menu"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ),
            create_button(&assets, "Start Game", MenuToGameButton)
        ],
    ));
}

fn setup_game_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SceneEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        children![
            (
                Text::new("Game Scene"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ),
            (
                Text::new("This is where your game would be!"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ),
            create_button(&assets, "Settings", GameToSettingsButton)
        ],
    ));
}

fn setup_settings_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SceneEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        children![
            (
                Text::new("Settings"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ),
            (
                Text::new("Configure your game settings here"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ),
            create_button(&assets, "Back to Menu", SettingsToMenuButton)
        ],
    ));
}

fn create_button(asset_server: &AssetServer, text: &str, button_type: impl Component) -> impl Bundle {
    (
        button_type,
        Button,
        Node {
            width: Val::Px(290.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::WHITE),
        BackgroundColor(NORMAL_BUTTON),
        children![(
            Text::new(text),
            TextFont {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )]
    )
}

fn cleanup_scene(mut commands: Commands, query: Query<Entity, With<SceneEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}