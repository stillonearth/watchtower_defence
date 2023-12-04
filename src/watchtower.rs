use crate::actions::Actions;
use crate::GameState;
use bevy::prelude::*;

pub struct WatchtowerPlugin;

#[derive(Component)]
pub struct Watchtower;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Watchtower`
impl Plugin for WatchtowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Watchtower), spawn_watchtower)
            .add_systems(Update, do_stuff.run_if(in_state(GameState::Watchtower)));
    }
}

fn spawn_watchtower(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands
    //     .spawn(SpriteBundle {
    //         texture: textures.bevy.clone(),
    //         transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
    //         ..Default::default()
    //     })
    //     .insert(Player);

    // note that we have to include the `Scene0` label
    let my_gltf = asset_server.load("models/scene.glb");

    println!("biboran");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn(SceneBundle {
        scene: my_gltf,
        transform: Transform::from_xyz(2.0, 0.0, -5.0),
        ..Default::default()
    });
}

fn do_stuff(
    time: Res<Time>,
    actions: Res<Actions>,
    mut watchtower_plugin: Query<&mut Transform, With<Watchtower>>,
) {
}
