use bevy::prelude::*;
use debug::DebugPlugin;
use bevy_egui::EguiPlugin;
use bevy::window::{WindowResolution, Window};
use munera::prelude::*;
use iso::{IsoCharacter, IsoPlugin, IsoCamera, Terrain};

fn main() {
    let mut app = App::new();

    app.add_plugins(MuneraPlugin)
        .add_plugins(IsoPlugin);

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(IsoCamera);
    commands.spawn(IsoCharacter);
    commands.spawn(Terrain {
        texture: asset_server.load("iso_color.png")
    });
}
