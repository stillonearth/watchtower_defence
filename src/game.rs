use std::time::Duration;

use crate::{events::*, loading::*, logic::*, ui::*, GameState};
use bevy::prelude::*;

use bevy_mod_picking::prelude::*;
use bevy_tweening::*;

// ----------
// Components
// ----------

#[derive(Component, Clone, Copy)]
pub struct Watchtower {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component)]
struct BoardCamera;

#[derive(Component)]
struct DebugSquare;

#[derive(Component)]
struct Square {
    pub i: usize,
    pub j: usize,
}

#[derive(Component)]
struct Circle {
    pub i: usize,
    pub j: usize,
}

#[derive(Resource)]
pub struct SelectedDraught {
    pub n: Option<i8>,
}

// -------
// Systems
// -------

fn spawn_go_piece(
    mut commands: Commands,
    q_squares: Query<(Entity, &Square)>,
    mut q_circles: Query<(Entity, &mut Visibility, &Circle)>,
) {
    for (entity, _) in q_squares.iter() {
        commands.entity(entity).remove::<PickableBundle>();
    }

    for (_, mut visibility, _) in q_circles.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

fn place_stone(
    mut commands: Commands,
    mut er_click_circle: EventReader<EventClickCircle>,
    q_circles: Query<(Entity, &mut Visibility, &Circle)>,
    meshes: Res<MeshAssets>,
    materials: Res<MaterialAssets>,
    mut turn: ResMut<Turn>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_logic: ResMut<GameLogic>,
    mut q_draughts: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_stones: Query<(Entity, &Stone)>,
    mut q_watchtowers: Query<(Entity, &Watchtower)>,
    mut q_debug_squares: Query<Entity, With<DebugSquare>>,
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

    let mut black_stones = q_stones
        .iter_mut()
        .filter(|(_, go_piece)| go_piece.side == Side::Black)
        .map(|go_piece| (go_piece.1.i, go_piece.1.j))
        .collect::<Vec<(usize, usize)>>();

    let mut white_stones = q_stones
        .iter_mut()
        .filter(|(_, go_piece)| go_piece.side == Side::White)
        .map(|go_piece| (go_piece.1.i, go_piece.1.j))
        .collect::<Vec<(usize, usize)>>();

    let white_watchtower = q_watchtowers
        .iter_mut()
        .find(|(_, watchtower)| watchtower.side == Side::White);
    if white_watchtower.is_none() {
        return;
    }
    let white_watchtower = white_watchtower.unwrap();
    let white_watchtower = (white_watchtower.1.i, white_watchtower.1.j);

    let black_watchtower = q_watchtowers
        .iter_mut()
        .find(|(_, watchtower)| watchtower.side == Side::Black);

    if black_watchtower.is_none() {
        return;
    }
    let black_watchtower = black_watchtower.unwrap();
    let black_watchtower = (black_watchtower.1.i, black_watchtower.1.j);

    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for er_click in er_click_circle.read() {
        let circle = q_circles.get_component::<Circle>(er_click.0).unwrap();
        commands.spawn((
            PbrBundle {
                mesh: meshes.go_piece.clone(),
                transform: Transform::from_translation(Vec3::new(
                    circle.i as f32 + 0.5,
                    0.06,
                    circle.j as f32 + 0.5,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                material: match side {
                    Side::Black => materials.blue.clone(),
                    _ => materials.yellow.clone(),
                },
                ..default()
            },
            Name::new("Stone"),
            Stone {
                i: circle.i,
                j: circle.j,
                side,
            },
        ));

        match side {
            Side::Black => black_stones.push((circle.i, circle.j)),
            Side::White => white_stones.push((circle.i, circle.j)),
        }

        let (convex_set, stone_removals) = game_logic.legal_go_moves(
            turn_,
            black_draughts.clone(),
            white_draughts.clone(),
            white_stones.clone(),
            black_stones.clone(),
            white_watchtower,
            black_watchtower,
        );

        // despawn debug squares
        for entity in q_debug_squares.iter_mut() {
            commands.entity(entity).despawn_recursive();
        }

        // remove opposite stones
        let enemy_stones = match side {
            Side::Black => white_stones.clone(),
            Side::White => black_stones.clone(),
        };

        let enemy_draughts = match side {
            Side::Black => white_draughts.clone(),
            Side::White => black_draughts.clone(),
        };

        for (i, j) in stone_removals.iter() {
            for enemy_stone in enemy_stones.iter() {
                if !(enemy_stone.0 == *i && enemy_stone.1 == *j) {
                    continue;
                }

                let stone_entity = q_stones
                    .iter()
                    .find(|(_, stone)| (stone.i == enemy_stone.0 && stone.j == enemy_stone.1));
                if stone_entity.is_some() {
                    let stone_entity = stone_entity.unwrap().0;
                    commands.entity(stone_entity).despawn_recursive();
                }
            }
        }

        for convex in convex_set.iter() {
            for (i, j) in convex.iter() {
                // takeover pieces
                for enemy_draught in enemy_draughts.iter() {
                    if !(enemy_draught.0 == *i && enemy_draught.1 == *j) {
                        continue;
                    }

                    let enemy_draught = q_draughts
                        .iter()
                        .find(|d| d.2.i == enemy_draught.0 && d.2.j == enemy_draught.1)
                        .unwrap();

                    let enemy_draught_entity = enemy_draught.0;
                    let enemy_draught = enemy_draught.2;
                    let n_draughts = q_draughts.iter().filter(|d| d.2.side == side).count();

                    commands.entity(enemy_draught_entity).despawn_recursive();

                    let transform =
                        Transform::from_xyz(enemy_draught.i as f32, 0.0, enemy_draught.j as f32)
                            .with_scale(Vec3::splat(0.1));
                    let draught = Draught {
                        i: enemy_draught.i,
                        j: enemy_draught.j,
                        n: (n_draughts + 1) as i8,
                        side,
                    };

                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.checkers_piece.clone(),
                            transform,
                            material: match side {
                                Side::Black => materials.black.clone(),
                                _ => materials.white.clone(),
                            },
                            ..default()
                        },
                        Name::new("Draught"),
                        draught,
                        On::<Pointer<Click>>::send_event::<EventClickDraught>(),
                    ));
                }

                // spawn debug square
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.square_plane.clone(),
                        material: materials.red.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            *i as f32, 0.0005, *j as f32,
                        )),
                        ..default()
                    },
                    Name::new("DebugSquare"),
                    DebugSquare,
                ));
            }
        }

        game_logic.log(GamePhase::PlaceGoPiece, *turn);
        let (next_phase, next_turn) = game_logic.next_state();
        *turn = next_turn;
        game_phase.set(next_phase);
        return;
    }
}

fn spawn_board(
    mut commands: Commands,
    materials: Res<MaterialAssets>,
    meshes: Res<MeshAssets>,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    // spawn checkerboard
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            let n = i * BOARD_SIZE + j;
            let material = match n % 2 {
                0 => materials.black.clone(),
                _ => materials.white.clone(),
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.square_plane.clone(),
                    material,
                    transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
                    ..default()
                },
                Square { i, j },
                Name::new("Square"),
                PickableBundle::default(),
                On::<Pointer<Over>>::send_event::<EventHoverSquare>(),
                On::<Pointer<Click>>::send_event::<EventClickSquare>(),
            ));
        }
    }

    // spawn go pieces intersections
    for i in 0..BOARD_SIZE - 1 {
        for j in 0..BOARD_SIZE - 1 {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.circle.clone(),
                    material: materials.red.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 + 0.5,
                        0.05,
                        j as f32 + 0.5,
                    ))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                Name::new("Circle"),
                Circle { i, j },
                PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<EventClickCircle>(),
            ));
        }
    }

    game_phase.set(GamePhase::PlaceWatchtower);
}

fn spawn_camera(mut commands: Commands) {
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(BOARD_SIZE as f32 / 2.0, 10.0, BOARD_SIZE as f32 / 2.0),
        ..Default::default()
    });

    let camera_transform = Transform::from_translation(Vec3::new(-5.0, 30.0, 20.0))
        .looking_at(Vec3::new(19.0 / 2.0, 0.0, 19.0 / 2.0), Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
        BoardCamera,
        Name::new("Camera"),
    ));
}

fn place_watchtower(
    mut commands: Commands,
    materials: Res<MaterialAssets>,
    mut er_hover_square: EventReader<EventHoverSquare>,
    mut er_click_square: EventReader<EventClickSquare>,
    _er_click_watchtower: EventReader<EventClickWatchtower>,
    q_squares: Query<Entity, &Square>,
    mut q_pieces: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_watchtower: Query<(Entity, &mut Transform, &mut Watchtower), Without<Draught>>,
    mut turn: ResMut<Turn>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_logic: ResMut<GameLogic>,
    meshes: Res<MeshAssets>,
) {
    let turn_ = *turn;

    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let mut stop = |center: (usize, usize)| -> Turn {
        let material = match side {
            Side::Black => materials.black.clone(),
            _ => materials.white.clone(),
        };

        for (entity, _, draught) in q_pieces.iter() {
            if draught.side != side {
                continue;
            }
            commands
                .entity(entity)
                .insert((PickableBundle::default(), material.clone()));
        }

        for (entity, _, watchtower) in q_watchtower.iter() {
            if watchtower.side != side {
                continue;
            }
            commands.entity(entity).insert(material.clone());
        }

        // place stones around watchtower
        let initial_stone_positions: Vec<(i32, i32)> = vec![
            (1, 1),
            (1, 0),
            (1, -1),
            (1, -2),
            (0, 1),
            (0, -2),
            (-1, 1),
            (-1, -2),
            (-2, 1),
            (-2, 0),
            (-2, -1),
            (-2, -2),
        ];

        for isp in initial_stone_positions {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.go_piece.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        (center.0 as i32 + isp.0) as f32 + 0.5,
                        0.05,
                        (center.1 as i32 + isp.1) as f32 + 0.5,
                    ))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    material: match side {
                        Side::Black => materials.blue.clone(),
                        _ => materials.yellow.clone(),
                    },
                    ..default()
                },
                Name::new("Stone"),
                Stone {
                    i: (center.0 as i32 + isp.0) as usize,
                    j: (center.1 as i32 + isp.1) as usize,
                    side,
                },
            ));
        }

        game_logic.log(GamePhase::PlaceWatchtower, turn_);
        let (next_phase, next_turn) = game_logic.next_state();
        if next_phase == GamePhase::PlaceWatchtower {
            game_phase.set(GamePhase::TriggerPlaceWatchtower);
        } else {
            game_phase.set(next_phase);
        }
        *turn = next_turn;

        next_turn
    };

    for click in er_click_square.read() {
        let watchtower = q_watchtower.get_component::<Watchtower>(click.0);

        if watchtower.is_ok() {
            let watchtower = watchtower.unwrap();
            let center = (watchtower.i, watchtower.j);
            *turn = stop(center);

            return;
        }
    }

    let opposite_side = match side {
        Side::Black => Side::White,
        _ => Side::Black,
    };

    let opposite_watchtower = q_watchtower
        .iter_mut()
        .find(|(_, _, watchtower)| watchtower.side == opposite_side);
    let mut opposite_watchtower_position = (-128.0, 128.0);
    if opposite_watchtower.is_some() {
        let opp_watchtower = opposite_watchtower.unwrap().2;
        opposite_watchtower_position = (opp_watchtower.i as f32, opp_watchtower.j as f32);
    }

    for hover in er_hover_square.read() {
        let square = q_squares.get_component::<Square>(hover.0).unwrap();
        let center = (square.i, square.j);

        for (_entity, mut transform, mut piece) in q_pieces.iter_mut() {
            if piece.side != side {
                continue;
            }

            // don't allow placing pieces on the edge
            if center.0 > 15 || center.1 > 15 || center.0 < 3 || center.1 < 3 {
                continue;
            }

            // don't allow placing pieces on the opposite watchtower
            let distance = ((center.0 as f32 - opposite_watchtower_position.0).powi(2)
                + (center.1 as f32 - opposite_watchtower_position.1).powi(2))
            .sqrt();

            if distance < 5.0 {
                continue;
            }

            match piece.n {
                0 => {
                    transform.translation.x = center.0 as f32 - 1.0;
                    transform.translation.z = center.1 as f32 - 1.0;
                    piece.i = center.0 - 1;
                    piece.j = center.1 - 1;
                }

                1 => {
                    transform.translation.x = center.0 as f32 - 1.0;
                    transform.translation.z = center.1 as f32;
                    piece.i = center.0 - 1;
                    piece.j = center.1;
                }

                2 => {
                    transform.translation.x = center.0 as f32 - 1.0;
                    transform.translation.z = center.1 as f32 + 1.0;
                    piece.i = center.0 - 1;
                    piece.j = center.1 + 1;
                }

                3 => {
                    transform.translation.x = center.0 as f32;
                    transform.translation.z = center.1 as f32 - 1.0;
                    piece.i = center.0;
                    piece.j = center.1 - 1;
                }

                4 => {
                    transform.translation.x = center.0 as f32;
                    transform.translation.z = center.1 as f32;
                    piece.i = center.0;
                    piece.j = center.1;
                }

                5 => {
                    transform.translation.x = center.0 as f32;
                    transform.translation.z = center.1 as f32 + 1.0;
                    piece.i = center.0;
                    piece.j = center.1 + 1;
                }

                6 => {
                    transform.translation.x = center.0 as f32 + 1.0;
                    transform.translation.z = center.1 as f32 - 1.0;
                    piece.i = center.0 + 1;
                    piece.j = center.1 - 1;
                }

                7 => {
                    transform.translation.x = center.0 as f32 + 1.0;
                    transform.translation.z = center.1 as f32;
                    piece.i = center.0 + 1;
                    piece.j = center.1;
                }

                8 => {
                    transform.translation.x = center.0 as f32 + 1.0;
                    transform.translation.z = center.1 as f32 + 1.0;
                    piece.i = center.0 + 1;
                    piece.j = center.1 + 1;
                }

                _ => {}
            }

            for (_entity, mut transform, mut watchtower) in q_watchtower.iter_mut() {
                if watchtower.side != side {
                    continue;
                }

                transform.translation.x = center.0 as f32;
                transform.translation.z = center.1 as f32;
                watchtower.i = center.0;
                watchtower.j = center.1;
            }
        }
    }
}

fn select_draught(
    mut er_click_draught: EventReader<EventClickDraught>,
    q_draughts: Query<(Entity, &mut Draught)>,
    mut selected_draught: ResMut<SelectedDraught>,
    mut q_nuke_draught_button: Query<(Entity, &mut Visibility, &ButtonNukeDraught)>,
    turn: Res<Turn>,
) {
    let side = match *turn.into_inner() {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for click in er_click_draught.read() {
        let draught = q_draughts.get_component::<Draught>(click.0).unwrap();

        if draught.side != side {
            return;
        }

        selected_draught.n = Some(draught.n);

        for (_, mut v, _) in q_nuke_draught_button.iter_mut() {
            *v = Visibility::Visible;
            break;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformPositionWithYJumpLens {
    pub start: Vec3,
    pub end: Vec3,
}

impl Lens<Transform> for TransformPositionWithYJumpLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let mut value = self.start + (self.end - self.start) * ratio;
        if ratio < 0.5 {
            value.y = ratio * 2.0 + 0.1;
        } else {
            value.y = (1.0 - ratio) * 2.0 + 0.1;
        }
        target.translation = value;
    }
}

fn move_draught(
    mut commands: Commands,
    mut er_click_square: EventReader<EventClickSquare>,
    mut q_draughts: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_stones: Query<&Stone>,
    mut q_watchtowers: Query<&Watchtower>,
    q_squares: Query<(Entity, &mut Transform, &mut Square), Without<Draught>>,
    mut selected_draught: ResMut<SelectedDraught>,
    mut turn: ResMut<Turn>,
    mut game_logic: ResMut<GameLogic>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    materials: Res<MaterialAssets>,
    meshes: Res<MeshAssets>,
    mut q_nuke_draught_button: Query<(Entity, &mut Visibility, &ButtonNukeDraught)>,
) {
    if selected_draught.n.is_none() {
        return;
    }

    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

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

    let white_watchtower = q_watchtowers
        .iter_mut()
        .find(|watchtower| watchtower.side == Side::White)
        .unwrap();
    let white_watchtower = (white_watchtower.i, white_watchtower.j);

    let black_watchtower = q_watchtowers
        .iter_mut()
        .find(|watchtower| watchtower.side == Side::Black)
        .unwrap();
    let black_watchtower = (black_watchtower.i, black_watchtower.j);

    for click in er_click_square.read() {
        let square = q_squares.get_component::<Square>(click.0);
        if square.is_err() {
            continue;
        }

        let square = q_squares.get_component::<Square>(click.0).unwrap();

        let draught = q_draughts
            .iter()
            .find(|d| d.2.n == selected_draught.n.unwrap() && d.2.side == side)
            .unwrap();

        let draught_position = draught.1.translation;
        let draught_entity = draught.0;
        let draught = draught.2;

        let (possible_moves, possible_movetypes, takeovers, _stone_removals) = game_logic
            .legal_draught_moves(
                turn_,
                (draught.i, draught.j),
                black_draughts,
                white_draughts,
                white_stones,
                black_stones,
                white_watchtower,
                black_watchtower,
            );

        if !possible_moves.contains(&(square.i, square.j)) {
            println!("Illegal move");
            return;
        }

        let possible_move_index = possible_moves
            .iter()
            .position(|(i, j)| *i == square.i && *j == square.j)
            .unwrap();
        let move_type = possible_movetypes[possible_move_index];

        if move_type == CheckersMoveType::DraughtTakeOver {
            let takeover = takeovers[possible_move_index];
            let enemy_draught = q_draughts
                .iter()
                .find(|d| d.2.i == takeover.0 && d.2.j == takeover.1)
                .unwrap();

            let enemy_draught_entity = enemy_draught.0;
            let enemy_draught = enemy_draught.2;
            let n_draughts = q_draughts.iter().filter(|d| d.2.side == side).count();

            commands.entity(enemy_draught_entity).despawn_recursive();

            let transform =
                Transform::from_xyz(enemy_draught.i as f32, 0.0, enemy_draught.j as f32)
                    .with_scale(Vec3::splat(0.1));
            let draught = Draught {
                i: enemy_draught.i,
                j: enemy_draught.j,
                n: (n_draughts + 1) as i8,
                side,
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.checkers_piece.clone(),
                    transform,
                    material: match side {
                        Side::Black => materials.black.clone(),
                        _ => materials.white.clone(),
                    },
                    ..default()
                },
                Name::new("Draught"),
                draught,
                On::<Pointer<Click>>::send_event::<EventClickDraught>(),
            ));
        }

        let square_position = q_squares
            .get_component::<Transform>(click.0)
            .unwrap()
            .translation;

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionWithYJumpLens {
                start: draught_position,
                end: square_position,
            },
        );

        commands.entity(draught_entity).insert((
            Animator::new(tween),
            Draught {
                i: square.i,
                j: square.j,
                ..*draught
            },
        ));

        selected_draught.n = None;

        for (_, mut v, _) in q_nuke_draught_button.iter_mut() {
            *v = Visibility::Hidden;
        }

        game_logic.log(GamePhase::MoveDraught, *turn);
        let (next_phase, next_turn) = game_logic.next_state();
        *turn = next_turn;
        game_phase.set(next_phase);

        return;
    }
}

fn prepare_move_draught(
    mut commands: Commands,
    mut q_circles: Query<(Entity, &mut Visibility, &Circle)>,
    mut q_draughts: Query<(Entity, &mut Draught)>,
    turn: ResMut<Turn>,
) {
    let turn = *turn;
    let side = match turn {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for (entity, mut visibility, _) in q_circles.iter_mut() {
        *visibility = Visibility::Hidden;
        commands.entity(entity).remove::<PickableBundle>();
    }

    for (entity, draught) in q_draughts.iter_mut() {
        if draught.side != side {
            commands.entity(entity).remove::<PickableBundle>();
        } else {
            commands.entity(entity).insert(PickableBundle::default());
        }
    }
}

fn spawn_watchtower(
    mut commands: Commands,
    meshes: Res<MeshAssets>,
    materials: Res<MaterialAssets>,
    turn: ResMut<Turn>,
    mut q_circles: Query<(Entity, &mut Visibility, &Circle)>,
) {
    for (entity, mut visibility, _) in q_circles.iter_mut() {
        *visibility = Visibility::Hidden;
        commands.entity(entity).remove::<PickableBundle>();
    }

    let center = (10.0, 10.0);

    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let material = match side {
        Side::Black => materials.transparent_black.clone(),
        _ => materials.transparent_white.clone(),
    };

    let transform = Transform::from_xyz(center.0, 0.9, center.1)
        .with_scale(Vec3::splat(0.1))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    commands.spawn((
        PbrBundle {
            mesh: meshes.watchtower.clone(),
            material: material.clone(),
            transform,
            ..default()
        },
        // PickableBundle::default(),
        Name::new("Watchtower"),
        Watchtower {
            i: center.0 as usize,
            j: center.1 as usize,
            side,
        },
        On::<Pointer<Click>>::send_event::<EventClickSquare>(),
    ));

    for i in 0..9 {
        if i == 4 {
            continue;
        }

        let piece_position = match i {
            0 => (center.0 - 1.0, center.1 - 1.0),
            1 => (center.0 - 1.0, center.1),
            2 => (center.0 - 1.0, center.1 + 1.0),

            3 => (center.0, center.1 - 1.0),
            4 => (center.0, center.1),
            5 => (center.0, center.1 + 1.0),

            6 => (center.0 + 1.0, center.1 - 1.0),
            7 => (center.0 + 1.0, center.1),
            8 => (center.0 + 1.0, center.1 + 1.0),

            _ => (0.0, 0.0),
        };

        let transform = Transform::from_xyz(piece_position.0, 0.0, piece_position.1)
            .with_scale(Vec3::splat(0.1));
        let draught = Draught {
            i: piece_position.0 as usize,
            j: piece_position.1 as usize,
            n: i,
            side,
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.checkers_piece.clone(),
                transform,
                material: material.clone(),
                ..default()
            },
            Name::new("Draught"),
            draught,
            On::<Pointer<Click>>::send_event::<EventClickDraught>(),
        ));
    }
}

fn check_game_termination(
    q_watchtowers: Query<(Entity, &Watchtower)>,
    mut text_query: Query<(&mut Text, &mut Visibility, &NextMoveText)>,
    mut next_game_phase: ResMut<NextState<GamePhase>>,
    current_game_phase: Res<State<GamePhase>>,
) {
    let game_phase = current_game_phase.get();
    if *game_phase == GamePhase::Initialize
        || *game_phase == GamePhase::PlaceWatchtower
        || *game_phase == GamePhase::TriggerPlaceWatchtower
    {
        return;
    }

    let n_white_towers = q_watchtowers
        .iter()
        .filter(|(_, w)| w.side == Side::White)
        .count();

    let n_black_towers = q_watchtowers
        .iter()
        .filter(|(_, w)| w.side == Side::Black)
        .count();

    if n_white_towers != 0 && n_white_towers != 0 {
        return;
    }

    let mut gameover_text = "";

    if n_white_towers == 0 {
        gameover_text = "Black Won!"
    }

    if n_black_towers == 0 {
        gameover_text = "White Won!"
    }

    for (mut text, mut v, _tag) in text_query.iter_mut() {
        text.sections[0].value = gameover_text.into();
        *v = Visibility::Visible;
    }

    next_game_phase.set(GamePhase::GameOver);
}

// --------------
// Game UI Plugin
// --------------

pub struct WatchtowerPlugin;

impl Plugin for WatchtowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GamePhase>();
        app.init_resource::<Turn>();
        app.add_plugins(DefaultPickingPlugins)
            .add_systems(Update, bevy_mod_picking::debug::hide_pointer_text)
            .add_systems(Startup, init_buttons)
            .add_systems(Update, nuke_draught_button_system)
            .add_systems(Startup, init_game_over_text)
            .add_systems(Update, check_game_termination)
            .add_systems(
                OnEnter(GameState::Watchtower),
                (spawn_camera, spawn_board, init_stats_text),
            )
            .add_systems(OnEnter(GamePhase::PlaceWatchtower), spawn_watchtower)
            .add_systems(
                OnEnter(GamePhase::TriggerPlaceWatchtower),
                |mut game_phase: ResMut<NextState<GamePhase>>| {
                    game_phase.set(GamePhase::PlaceWatchtower);
                },
            )
            .add_systems(
                OnEnter(GamePhase::PlaceGoPiece),
                (spawn_go_piece, show_stats),
            )
            .add_systems(
                OnEnter(GamePhase::MoveDraught),
                (prepare_move_draught, show_stats),
            )
            .add_systems(
                Update,
                place_watchtower.run_if(in_state(GamePhase::PlaceWatchtower)),
            )
            .add_systems(
                Update,
                place_stone.run_if(in_state(GamePhase::PlaceGoPiece)),
            )
            .add_systems(
                Update,
                (select_draught, move_draught).run_if(in_state(GamePhase::MoveDraught)),
            )
            .add_event::<EventHoverSquare>()
            .add_event::<EventClickWatchtower>()
            .add_event::<EventClickSquare>()
            .add_event::<EventClickCircle>()
            .add_event::<EventClickDraught>()
            .insert_resource(GameLogic::new())
            .insert_resource(SelectedDraught { n: None })
            .insert_resource(ClearColor(Color::BLACK));
    }
}
