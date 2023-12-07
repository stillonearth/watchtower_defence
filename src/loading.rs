use crate::{watchtower, GameState};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        info!("here!");

        // let my_gltf = ass.load("my.glb#Scene0");

        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Watchtower),
        );
        app.add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading);
        app.add_collection_to_loading_state::<_, TextureAssets>(GameState::Loading);

        app.init_resource::<MaterialAssets>();
        app.init_resource::<MeshAssets>();
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
}

#[derive(Resource)]
pub struct MeshAssets {
    pub square_plane: Handle<Mesh>,
    pub checkers_piece: Handle<Mesh>,
    pub watchtower: Handle<Mesh>,
}

impl FromWorld for MeshAssets {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        MeshAssets {
            square_plane: meshes.add(Mesh::from(shape::Plane {
                size: 1.,
                ..default()
            })),
            checkers_piece: meshes.add(Mesh::from(shape::Cylinder {
                radius: 3.0,
                height: 2.0,
                resolution: 32,
                ..default()
            })),
            watchtower: asset_server.load("models/watchtower.glb#Mesh0/Primitive0"),
        }
    }
}

#[derive(Resource)]
pub struct MaterialAssets {
    pub black: Handle<StandardMaterial>,
    pub white: Handle<StandardMaterial>,
    pub transparent_white: Handle<StandardMaterial>,
    pub transparent_black: Handle<StandardMaterial>,
}

impl FromWorld for MaterialAssets {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials_asset = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        MaterialAssets {
            black: materials_asset.add(bevy::prelude::Color::rgb(0., 0.1, 0.1).into()),
            white: materials_asset.add(bevy::prelude::Color::rgb(1., 0.9, 0.9).into()),
            transparent_white: materials_asset
                .add(bevy::prelude::Color::rgba(1., 0.9, 0.9, 0.5).into()),
            transparent_black: materials_asset
                .add(bevy::prelude::Color::rgba(0., 0.1, 0.1, 0.5).into()),
        }
    }
}
