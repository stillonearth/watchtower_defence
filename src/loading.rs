use crate::GameState;
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
        app.add_collection_to_loading_state::<_, ModelAssets>(GameState::Loading);

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

#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models/watchtower.glb")]
    pub watchtower: Handle<Scene>,
}

#[derive(Resource)]
pub struct MeshAssets {
    pub square_plane: Handle<Mesh>,
}

impl FromWorld for MeshAssets {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();

        MeshAssets {
            square_plane: meshes.add(Mesh::from(shape::Plane {
                size: 1.,
                ..default()
            })),
        }
    }
}

#[derive(Resource)]
pub struct MaterialAssets {
    pub black: Handle<StandardMaterial>,
    pub white: Handle<StandardMaterial>,
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
        }
    }
}
