use std::time::Duration;

use crate::loading::{MaterialAssets, MeshAssets, TextureAssets};
use crate::GameState;
use bevy::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::*;
use rand::Rng;

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(GameState::Menu),
            |mut commands: Commands, q_menu_components: Query<(Entity, &MenuComponent)>| {
                for (e, _) in q_menu_components.iter() {
                    commands.entity(e).despawn_recursive();
                }
            },
        )
        .add_systems(OnEnter(GameState::Menu), (setup_menu, spawn_board))
        .add_systems(Update, click_play_button.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb(0.15, 0.15, 0.15),
            hovered: Color::rgb(0.25, 0.25, 0.25),
        }
    }
}

#[derive(Component)]
struct MenuComponent;

#[derive(Component)]
struct MenuGamePiece;

const BOARD_SIZE: usize = 257;

fn spawn_board(mut commands: Commands, materials: Res<MaterialAssets>, meshes: Res<MeshAssets>) {
    let mut rng = rand::thread_rng();
    // Light
    const N_LIGHTS: usize = 15;
    for i in 0..N_LIGHTS {
        for j in 0..N_LIGHTS {
            let start_light: f64 = rng.gen_range(10.0..50.0);
            let end_light: f64 = rng.gen_range(1.0..10.0);
            let duration: f64 = rng.gen_range(1.0..30.0);

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(duration as u64),
                TransformPositionLens {
                    start: Vec3::new(
                        (BOARD_SIZE as f32 / N_LIGHTS as f32) * (i as f32),
                        start_light as f32,
                        (BOARD_SIZE as f32 / N_LIGHTS as f32) * (j as f32),
                    ),
                    end: Vec3::new(
                        (BOARD_SIZE as f32 / N_LIGHTS as f32) * (i as f32),
                        end_light as f32,
                        (BOARD_SIZE as f32 / N_LIGHTS as f32) * (j as f32),
                    ),
                },
            )
            .with_repeat_count(RepeatCount::Infinite)
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

            commands.spawn((
                PointLightBundle {
                    point_light: PointLight {
                        intensity: 2000.0,
                        shadows_enabled: false,
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(
                        (BOARD_SIZE as f32 / N_LIGHTS as f32) * (i as f32),
                        start_light as f32,
                        (BOARD_SIZE as f32 / N_LIGHTS as f32) * (j as f32),
                    ),
                    ..Default::default()
                },
                Animator::new(tween),
                MenuComponent,
            ));
        }
    }

    let camera_transform = Transform::from_translation(Vec3::new(
        (BOARD_SIZE as f32) / 2.,
        (BOARD_SIZE as f32) / 4.,
        (BOARD_SIZE as f32) / 2.,
    ))
    .looking_at(Vec3::new(94., 0.0, 121.), Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
        Name::new("Camera"),
        MenuComponent,
    ));

    // spawn checkerboard
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            let n = i * BOARD_SIZE + j;
            let material = match n % 2 {
                0 => materials.black.clone(),
                _ => materials.white.clone(),
            };

            let initial_height: f64 = rng.gen_range(-100.0..100.0);
            let duration: f64 = rng.gen_range(1.0..5.0);

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(duration as u64),
                TransformPositionLens {
                    start: Vec3::new(i as f32, initial_height as f32, j as f32),
                    end: Vec3::new(i as f32, 0 as f32, j as f32),
                },
            );

            commands.spawn((
                PbrBundle {
                    mesh: meshes.square_plane.clone(),
                    material,
                    transform: Transform::from_translation(Vec3::new(
                        i as f32,
                        initial_height as f32,
                        j as f32,
                    )),
                    ..default()
                },
                Name::new("Square"),
                Animator::new(tween),
                MenuComponent,
            ));

            // spawn go piece or checker randomly
            let dice_roll = rng.gen_range(0..4) as usize;

            match dice_roll {
                // stone
                0 => {
                    let color = rng.gen_range(0..2) as usize;

                    let material = match color {
                        0 => materials.black.clone(),
                        _ => materials.white.clone(),
                    };

                    let transform =
                        Transform::from_xyz(i as f32, 0.0, j as f32).with_scale(Vec3::splat(0.1));

                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.checkers_piece.clone(),
                            transform,
                            material,
                            ..default()
                        },
                        Name::new("Draught"),
                        MenuGamePiece,
                        MenuComponent,
                    ));
                }
                // draught
                1 => {
                    let color = rng.gen_range(0..2) as usize;

                    let material = match color {
                        0 => materials.yellow.clone(),
                        _ => materials.blue.clone(),
                    };

                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.go_piece.clone(),
                            transform: Transform::from_translation(Vec3::new(
                                i as f32 + 0.5,
                                0.0002,
                                j as f32 + 0.5,
                            ))
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                            material,
                            ..default()
                        },
                        Name::new("Stone"),
                        MenuGamePiece,
                        MenuComponent,
                    ));
                }
                _ => {}
            }
        }
    }
}

#[derive(Component)]
struct Menu;

fn setup_menu(mut commands: Commands, textures: Res<TextureAssets>) {
    info!("menu");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            let button_colors = ButtonColors::default();
            children
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(140.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: button_colors.normal.into(),
                        ..Default::default()
                    },
                    button_colors,
                    ChangeState(GameState::Watchtower),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    bottom: Val::Px(5.),
                    width: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            children
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(170.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(5.)),
                            ..Default::default()
                        },
                        background_color: Color::NONE.into(),
                        ..Default::default()
                    },
                    ButtonColors {
                        normal: Color::NONE,
                        ..default()
                    },
                    OpenLink("https://bevyengine.org"),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Made with Bevy for Bevy Jam #4",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                    parent.spawn(ImageBundle {
                        image: textures.bevy.clone().into(),
                        style: Style {
                            width: Val::Px(32.),
                            ..default()
                        },
                        ..default()
                    });
                });
            children
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(170.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(5.)),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..Default::default()
                    },
                    ButtonColors {
                        normal: Color::NONE,
                        hovered: Color::rgb(0.25, 0.25, 0.25),
                    },
                    OpenLink("https://github.com/stillonearth/watchtower_defence"),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Open source",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                    parent.spawn(ImageBundle {
                        image: textures.github.clone().into(),
                        style: Style {
                            width: Val::Px(32.),
                            ..default()
                        },
                        ..default()
                    });
                });
        });
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
            Option<&OpenLink>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button_colors, change_state, open_link) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(state) = change_state {
                    next_state.set(state.0.clone());
                } else if let Some(link) = open_link {
                    if let Err(error) = webbrowser::open(link.0) {
                        warn!("Failed to open link {error:?}");
                    }
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
