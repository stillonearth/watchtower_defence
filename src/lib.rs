#![allow(clippy::type_complexity)]

mod audio;
mod events;
mod game;
mod loading;
mod logic;
mod menu;
mod ui;

use crate::audio::InternalAudioPlugin;
use crate::game::WatchtowerPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use bevy::app::App;

use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Watchtower,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        info!("here");

        app.add_state::<GameState>();
        app.add_plugins((
            LoadingPlugin,
            MenuPlugin,
            InternalAudioPlugin,
            WatchtowerPlugin,
            TweeningPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            // app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
