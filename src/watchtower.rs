use crate::loading::{MaterialAssets, MeshAssets};
use crate::GameState;
use bevy_mod_picking::prelude::*;

use bevy::prelude::*;

pub struct WatchtowerPlugin;

#[derive(Component)]
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
}

impl Plugin for WatchtowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GamePhase>();
        app.init_resource::<Turn>();
        app.add_plugins(DefaultPickingPlugins)
            .add_systems(OnEnter(GameState::Watchtower), (spawn_camera, spawn_board))
            .add_systems(OnEnter(GamePhase::PlaceWatchtower), (spawn_watchtower))
            .add_systems(OnEnter(GamePhase::PlaceGoPiece), (spawn_go_piece))
            .add_systems(
                Update,
                place_watchtower.run_if(in_state(GamePhase::PlaceWatchtower)),
            )
            .add_systems(
                Update,
                place_go_piece.run_if(in_state(GamePhase::PlaceGoPiece)),
            )
            .add_event::<EventHoverSquare>()
            .add_event::<EventClickWatchtower>()
            .add_event::<EventClickSquare>()
            .add_event::<EventClickCircle>()
            .insert_resource(GameLogic::new());
    }
}

fn spawn_watchtower(
    mut commands: Commands,
    meshes: Res<MeshAssets>,
    materials: Res<MaterialAssets>,
    mut turn: ResMut<Turn>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_logic: ResMut<GameLogic>,
) {
    let center = (10.0, 10.0);

    let turn_ = turn.clone();
    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let transform = Transform::from_xyz(center.0, 0.9, center.1)
        .with_scale(Vec3::splat(0.1))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    commands.spawn((
        PbrBundle {
            mesh: meshes.watchtower.clone(),
            material: materials.transparent_white.clone(),
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
        let piece = Draught {
            i: piece_position.0 as usize,
            j: piece_position.1 as usize,
            n: i,
            side: side.clone(),
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.checkers_piece.clone(),
                transform,
                material: materials.transparent_white.clone(),
                ..default()
            },
            Name::new("Piece"),
            piece,
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
    let turn_ = turn.clone();
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
            PickableBundle::default(),
            GoPiece {
                i: circle.i,
                j: circle.j,
                side,
            },
        ));

        game_logic.log(GamePhase::PlaceGoPiece, turn.clone());
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

    let camera_transform = Transform::from_translation(Vec3::new(-5.0, 20.0, 20.0))
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
    let turn_ = turn.clone();

    let side = match turn_ {
        Turn::Black => Side::Black,
        Turn::White => Side::White,
    };

    let mut stop = || -> Turn {
        let material = match side {
            Side::Black => materials.black.clone(),
            _ => materials.white.clone(),
        };

        for (entity, _, _) in q_pieces.iter() {
            commands
                .entity(entity)
                .insert((PickableBundle::default(), material.clone()));
        }

        for (entity, _, _) in q_watchtower.iter() {
            commands
                .entity(entity)
                .insert((PickableBundle::default(), material.clone()));
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

    for hover in er_hover_square.read() {
        let square = q_squares.get_component::<Square>(hover.0).unwrap();
        let center = (square.i, square.j);

        for (_entity, mut transform, mut piece) in q_pieces.iter_mut() {
            if piece.side != side {
                continue;
            }

            if (center.0 == 18 || center.1 == 18 || center.0 == 0 || center.1 == 0) {
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
