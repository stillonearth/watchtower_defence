use crate::actions::Actions;
use crate::loading::{MaterialAssets, MeshAssets};
use crate::GameState;
use bevy_mod_picking::prelude::*;

use bevy::prelude::*;

pub struct WatchtowerPlugin;

#[derive(Component)]
pub struct Watchtower;

#[derive(Component)]
struct BoardCamera;

#[derive(Component)]
struct Square {
    pub i: usize,
    pub j: usize,
}

#[derive(Component)]
struct Piece {
    pub i: usize,
    pub j: usize,
}

const BOARD_SIZE: usize = 19;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GamePhase {
    #[default]
    Initialize,
    PlaceWatchtower,
    PlaceGoPiece,
    MoveCheckersPiece,
}

#[derive(Resource, Default)]
enum Turn {
    Black,
    #[default]
    White,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Watchtower`
impl Plugin for WatchtowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GamePhase>();
        app.init_resource::<Turn>();
        app.add_plugins(DefaultPickingPlugins)
            .add_systems(OnEnter(GameState::Watchtower), (spawn_camera, spawn_board))
            .add_systems(OnEnter(GamePhase::PlaceWatchtower), (spawn_watchtower))
            .add_systems(
                Update,
                place_watchtower.run_if(in_state(GamePhase::PlaceWatchtower)),
            );
    }
}

fn spawn_watchtower(
    mut commands: Commands,
    meshes: Res<MeshAssets>,
    materials: Res<MaterialAssets>,
) {
    let watchtower_position = (8.0, 8.0);

    let transform = Transform::from_xyz(watchtower_position.0, 0.9, watchtower_position.1)
        .with_scale(Vec3::splat(0.1))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    commands.spawn((
        PbrBundle {
            mesh: meshes.watchtower.clone(),
            material: materials.transparent_white.clone(),
            transform,
            ..default()
        },
        PickableBundle::default(),
        Name::new("Watchtower"),
        Watchtower,
    ));

    for i in 0..9 {
        if i == 4 {
            continue;
        }

        let piece_position = match i {
            0 => (watchtower_position.0 - 1.0, watchtower_position.1 - 1.0),
            1 => (watchtower_position.0 - 1.0, watchtower_position.1),
            2 => (watchtower_position.0 - 1.0, watchtower_position.1 + 1.0),

            3 => (watchtower_position.0, watchtower_position.1 - 1.0),
            4 => (watchtower_position.0, watchtower_position.1),
            5 => (watchtower_position.0, watchtower_position.1 + 1.0),

            6 => (watchtower_position.0 + 1.0, watchtower_position.1 - 1.0),
            7 => (watchtower_position.0 + 1.0, watchtower_position.1),
            8 => (watchtower_position.0 + 1.0, watchtower_position.1 + 1.0),

            _ => (0.0, 0.0),
        };

        let transform = Transform::from_xyz(piece_position.0, 0.0, piece_position.1)
            .with_scale(Vec3::splat(0.1));
        let square = Square {
            i: piece_position.0 as usize,
            j: piece_position.1 as usize,
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.checkers_piece.clone(),
                transform,
                material: materials.transparent_white.clone(),
                ..default()
            },
            Name::new("Watchtower"),
            PickableBundle::default(),
            square,
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

fn spawn_board(
    mut commands: Commands,
    materials: Res<MaterialAssets>,
    meshes: Res<MeshAssets>,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
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
        transform: Transform::from_xyz(10.0, 4.0, 9.0),
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
    time: Res<Time>,
    actions: Res<Actions>,
    mut watchtower_plugin: Query<&mut Transform, With<Watchtower>>,
    er_hover_square: EventReader<EventHoverSquare>,
    q_squares: Query<Entity, Square>,
) {
    for 
}
