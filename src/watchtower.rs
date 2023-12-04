use crate::actions::Actions;
use crate::loading::{MaterialAssets, MeshAssets, ModelAssets};
use crate::GameState;

use bevy::prelude::*;

pub struct WatchtowerPlugin;

#[derive(Component)]
pub struct Watchtower;

#[derive(Component)]
struct BoardCamera;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Watchtower`
impl Plugin for WatchtowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Watchtower),
            (spawn_watchtower, spawn_camera, spawn_board),
        )
        .add_systems(Update, do_stuff.run_if(in_state(GameState::Watchtower)));
    }
}

fn spawn_watchtower(mut commands: Commands, models: Res<ModelAssets>) {
    let transform = Transform::from_xyz(2.0, 0.9, -5.0).with_scale(Vec3::splat(0.1));
    commands.spawn((
        SceneBundle {
            scene: models.watchtower.clone(),
            transform,
            ..default()
        },
        Name::new("Watchtower"),
    ));
}

const BOARD_SIZE: usize = 19;

fn spawn_board(mut commands: Commands, materials: Res<MaterialAssets>, meshes: Res<MeshAssets>) {
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
                Name::new("Square"),
            ));
        }
    }
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

    let camera_transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
        Vec3::new(-8.5, 30.0, 9.0),
    ))
    .with_scale(Vec3::new(1.0, 1.0, 1.5));

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
        BoardCamera,
        Name::new("Camera"),
    ));
}

fn do_stuff(
    time: Res<Time>,
    actions: Res<Actions>,
    mut watchtower_plugin: Query<&mut Transform, With<Watchtower>>,
) {
}
