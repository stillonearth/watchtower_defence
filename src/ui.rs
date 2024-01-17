use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{game::*, logic::*};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct NextMoveText;

#[derive(Component)]
pub struct ButtonNukeDraught;

#[derive(Component)]
pub struct GameOverText;

#[derive(Component)]
pub struct GameStatsText;

pub fn init_stats_text(mut commands: Commands) {
    let text = Text::from_section(
        "",
        TextStyle {
            font_size: 15.0,
            color: Color::WHITE,
            ..default()
        },
    )
    .with_alignment(TextAlignment::Left);

    // root node
    commands
        .spawn((NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Percent(10.),
                top: Val::Percent(10.),
                width: Val::Px(250.0),
                ..Default::default()
            },
            ..Default::default()
        },))
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text,
                    ..Default::default()
                })
                .insert((GameStatsText, Visibility::Visible));
        })
        .insert(Pickable::IGNORE);
}

pub fn show_stats(
    mut q_draughts: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_stones: Query<&Stone>,
    game_logic: ResMut<GameLogic>,
    mut text_query: Query<(&mut Text, &GameStatsText)>,
) {
    let black_draughts = q_draughts
        .iter_mut()
        .filter(|(_, _, draught)| draught.side == Side::Black)
        .map(|(_, _, draught)| (draught.i, draught.j))
        .collect::<Vec<(usize, usize)>>();

    let white_draughts = q_draughts
        .iter_mut()
        .filter(|(_, _, draught)| draught.side == Side::White)
        .map(|(_, _, draught)| (draught.i, draught.j))
        .collect::<Vec<(usize, usize)>>();

    let black_stones = q_stones
        .iter_mut()
        .filter(|go_piece| go_piece.side == Side::Black)
        .map(|go_piece| (go_piece.i, go_piece.j))
        .collect::<Vec<(usize, usize)>>();

    let white_stones = q_stones
        .iter_mut()
        .filter(|go_piece| go_piece.side == Side::White)
        .map(|go_piece| (go_piece.i, go_piece.j))
        .collect::<Vec<(usize, usize)>>();

    let stats = game_logic.stats(black_draughts, white_draughts, white_stones, black_stones);

    for (mut text, _tag) in text_query.iter_mut() {
        text.sections[0].value = format!("{:?}", stats);
    }
}

pub fn init_buttons(mut commands: Commands) {
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(170.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: NORMAL_BUTTON.into(),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            ButtonNukeDraught,
            // Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Nuke the Draught",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                },
                // Visibility::Hidden,
            ));
        })
        .insert(Pickable::IGNORE);
}

#[allow(clippy::type_complexity)]
pub fn nuke_draught_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&ButtonNukeDraught, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    q_draughts: Query<(Entity, &mut Draught)>,
    q_stones: Query<(Entity, &Stone)>,
    q_watchtowers: Query<(Entity, &Watchtower)>,
    mut selected_draught: ResMut<SelectedDraught>,
    mut q_nuke_draught_button: Query<(Entity, &mut Visibility, &ButtonNukeDraught)>,
    mut turn: ResMut<Turn>,
    mut game_logic: ResMut<GameLogic>,

    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for (_, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                let draught = q_draughts
                    .iter()
                    .find(|d| d.1.n == selected_draught.n.unwrap() && d.1.side == side)
                    .unwrap()
                    .1;

                let ccs_to_remove: Vec<(i32, i32)> = vec![
                    (draught.i as i32 - 1, draught.j as i32 - 1),
                    (draught.i as i32 - 1, draught.j as i32),
                    (draught.i as i32 - 1, draught.j as i32 + 1),
                    (draught.i as i32, draught.j as i32 - 1),
                    (draught.i as i32, draught.j as i32),
                    (draught.i as i32, draught.j as i32 + 1),
                    (draught.i as i32 + 1, draught.j as i32 - 1),
                    (draught.i as i32 + 1, draught.j as i32),
                    (draught.i as i32 + 1, draught.j as i32 + 1),
                ];

                let ccs_to_remove: Vec<_> = ccs_to_remove
                    .iter()
                    .filter(|(i, j)| *i >= 0 && *j >= 0)
                    .collect();

                for (e, d) in q_draughts.iter() {
                    for cc in ccs_to_remove.iter() {
                        if d.i as i32 == cc.0 && d.j as i32 == cc.1 {
                            commands.entity(e).despawn_recursive();
                        }
                    }
                }

                for (e, d) in q_stones.iter() {
                    for cc in ccs_to_remove.iter() {
                        if d.i as i32 == cc.0 && d.j as i32 == cc.1 {
                            commands.entity(e).despawn_recursive();
                        }
                    }
                }

                for (e, d) in q_watchtowers.iter() {
                    for cc in ccs_to_remove.iter() {
                        if d.i as i32 == cc.0 && d.j as i32 == cc.1 {
                            commands.entity(e).despawn_recursive();
                        }
                    }
                }

                selected_draught.n = None;

                for (_, mut v, _) in q_nuke_draught_button.iter_mut() {
                    *v = Visibility::Hidden;
                }

                *color = PRESSED_BUTTON.into();

                game_logic.log(GamePhase::MoveDraught, turn_);
                let (next_phase, next_turn) = game_logic.next_state();
                *turn = next_turn;
                game_phase.set(next_phase);
                return;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn init_game_over_text(mut commands: Commands) {
    let text = Text::from_section(
        "GAME OVER",
        TextStyle {
            font_size: 50.0,
            color: Color::WHITE,
            ..default()
        },
    )
    .with_alignment(TextAlignment::Left);

    // root node
    commands
        .spawn((NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.),
                bottom: Val::Percent(50.),
                ..Default::default()
            },
            ..Default::default()
        },))
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text,
                    ..Default::default()
                })
                .insert((NextMoveText, Visibility::Hidden));
        })
        .insert(Pickable::IGNORE);
}
