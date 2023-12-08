use std::time::Duration;

use crate::loading::{MaterialAssets, MeshAssets};
use crate::GameState;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_tweening::*;

pub struct WatchtowerPlugin;

#[derive(Component, Clone, Copy)]
pub struct Watchtower {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component)]
struct BoardCamera;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Side {
    Black,
    White,
    Neutral,
}

#[derive(Component)]
struct Square {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component)]
struct Circle {
    pub i: usize,
    pub j: usize,
}

#[derive(Component)]
struct GoPiece {
    pub i: usize,
    pub j: usize,
    pub side: Side,
}

#[derive(Component, Debug)]
struct Draught {
    pub i: usize,
    pub j: usize,
    pub n: usize,
    pub side: Side,
}

const BOARD_SIZE: usize = 19;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GamePhase {
    #[default]
    Initialize,
    PlaceWatchtower,
    PlaceGoPiece,
    MoveDraught,
}

#[derive(Resource, Default, Clone, Copy)]
enum Turn {
    Black,
    #[default]
    White,
}

#[derive(Resource)]
struct GameLogic {
    log: Vec<(GamePhase, Turn)>,
}

#[derive(Resource)]
struct SelectedDraught {
    n: Option<usize>,
}

impl GameLogic {
    fn new() -> Self {
        GameLogic { log: vec![] }
    }

    fn log(&mut self, game_phase: GamePhase, turn: Turn) {
        self.log.push((game_phase, turn));
    }

    fn next_state(&self) -> (GamePhase, Turn) {
        let (game_phase, turn) = self.log.last().unwrap();
        match game_phase {
            GamePhase::PlaceWatchtower => (GamePhase::PlaceGoPiece, *turn),
            GamePhase::PlaceGoPiece => {
                if self.log.len() == 2 {
                    return (GamePhase::PlaceWatchtower, Turn::Black);
                }
                (
                    GamePhase::MoveDraught,
                    match turn {
                        Turn::Black => Turn::White,
                        Turn::White => Turn::Black,
                    },
                )
            }
            GamePhase::MoveDraught => (GamePhase::PlaceGoPiece, *turn),
            _ => (GamePhase::PlaceWatchtower, *turn),
        }
    }

    fn legal_draught_moves(
        &self,
        turn: Turn,
        draught: (usize, usize),
        black_draughts: Vec<(usize, usize)>,
        white_draughts: Vec<(usize, usize)>,
        white_stones: Vec<(usize, usize)>,
        black_stones: Vec<(usize, usize)>,
        white_tower: (usize, usize),
        black_tower: (usize, usize),
    ) {
        let ((our_draughts, our_stones, our_tower), (enemy_draughts, enemy_stones, enemy_tower)) =
            match turn {
                Turn::Black => (
                    (black_draughts, black_stones, black_tower),
                    (white_draughts, white_stones, white_tower),
                ),
                Turn::White => (
                    (white_draughts, white_stones, white_tower),
                    (black_draughts, black_stones, black_tower),
                ),
            };

        let legal_moves: Vec<(usize, usize)> = Vec::new();
        let occupied_squares = 
    }
}

impl Plugin for WatchtowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GamePhase>();
        app.init_resource::<Turn>();
        app.add_plugins(TweeningPlugin)
            .add_plugins(DefaultPickingPlugins)
            .add_systems(OnEnter(GameState::Watchtower), (spawn_camera, spawn_board))
            .add_systems(OnEnter(GamePhase::PlaceWatchtower), spawn_watchtower)
            .add_systems(OnEnter(GamePhase::PlaceGoPiece), spawn_go_piece)
            .add_systems(OnEnter(GamePhase::MoveDraught), prepare_move_draught)
            .add_systems(
                Update,
                place_watchtower.run_if(in_state(GamePhase::PlaceWatchtower)),
            )
            .add_systems(
                Update,
                place_go_piece.run_if(in_state(GamePhase::PlaceGoPiece)),
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
            .insert_resource(SelectedDraught { n: None });
    }
}

fn select_draught(
    mut er_click_draught: EventReader<EventClickDraught>,
    q_draughts: Query<(Entity, &mut Draught)>,
    mut selected_draught: ResMut<SelectedDraught>,
) {
    for click in er_click_draught.read() {
        let draught = q_draughts.get_component::<Draught>(click.0).unwrap();
        selected_draught.n = Some(draught.n);
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
    q_draughts: Query<(Entity, &mut Transform, &mut Draught)>,
    q_squares: Query<(Entity, &mut Transform, &mut Square), Without<Draught>>,
    mut selected_draught: ResMut<SelectedDraught>,
    mut turn: ResMut<Turn>,
    mut game_logic: ResMut<GameLogic>,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    if selected_draught.n.is_none() {
        return;
    }

    let turn_ = *turn;
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    for click in er_click_square.read() {
        let square_position = q_squares
            .get_component::<Transform>(click.0)
            .unwrap()
            .translation;

        let draught = q_draughts
            .iter()
            .find(|d| d.2.n == selected_draught.n.unwrap() && d.2.side == side)
            .unwrap();

        let draught_position = draught.1.translation;
        let draught_entity = draught.0;

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionWithYJumpLens {
                start: draught_position,
                end: square_position,
            },
        );

        commands.entity(draught_entity).insert(Animator::new(tween));

        selected_draught.n = None;

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

#[derive(Event)]
struct EventHoverSquare(Entity);

impl From<ListenerInput<Pointer<Over>>> for EventHoverSquare {
    fn from(event: ListenerInput<Pointer<Over>>) -> Self {
        EventHoverSquare(event.target)
    }
}

#[derive(Event)]
struct EventClickSquare(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickSquare {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickSquare(event.target)
    }
}

#[derive(Event)]
struct EventClickDraught(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickDraught {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickDraught(event.target)
    }
}

#[derive(Event)]
struct EventClickCircle(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickCircle {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickCircle(event.target)
    }
}

#[derive(Event)]
struct EventClickWatchtower(Entity);

impl From<ListenerInput<Pointer<Click>>> for EventClickWatchtower {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EventClickWatchtower(event.target)
    }
}

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

fn place_go_piece(
    mut commands: Commands,
    mut er_click_circle: EventReader<EventClickCircle>,
    q_circles: Query<(Entity, &mut Visibility, &Circle)>,
    meshes: Res<MeshAssets>,
    materials: Res<MaterialAssets>,
    mut turn: ResMut<Turn>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_logic: ResMut<GameLogic>,
) {
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
                    0.0002,
                    circle.j as f32 + 0.5,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                material: match side {
                    Side::Black => materials.black.clone(),
                    _ => materials.white.clone(),
                },
                ..default()
            },
            Name::new("GoPiece"),
            GoPiece {
                i: circle.i,
                j: circle.j,
                side,
            },
        ));

        game_logic.log(GamePhase::PlaceGoPiece, *turn);
        let (next_phase, next_turn) = game_logic.next_state();
        *turn = next_turn;
        game_phase.set(next_phase);
    }
}

fn spawn_board(
    mut commands: Commands,
    materials: Res<MaterialAssets>,
    meshes: Res<MeshAssets>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    turn: Res<Turn>,
) {
    let side = match *turn.into_inner() {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

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
                Square { i, j, side },
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
                        0.0001,
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
    mut er_click_watchtower: EventReader<EventClickWatchtower>,
    q_squares: Query<Entity, &Square>,
    mut q_pieces: Query<(Entity, &mut Transform, &mut Draught)>,
    mut q_watchtower: Query<(Entity, &mut Transform, &mut Watchtower), Without<Draught>>,
    mut turn: ResMut<Turn>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_logic: ResMut<GameLogic>,
) {
    let turn_ = *turn;

    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let mut stop = || -> Turn {
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

        game_logic.log(GamePhase::PlaceWatchtower, turn_);
        let (next_phase, next_turn) = game_logic.next_state();
        game_phase.set(next_phase);

        next_turn
    };

    for _click in er_click_square.read() {
        *turn = stop();
        return;
    }

    for _click in er_click_watchtower.read() {
        *turn = stop();
        return;
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

            if distance < 8.0 {
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
