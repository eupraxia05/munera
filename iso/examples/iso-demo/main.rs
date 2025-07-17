use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_egui::EguiPlugin;

use iso::{IsoCharacter, IsoPlugin, IsoCamera, Terrain};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(IsoPlugin);

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn(IsoCamera);
    commands.spawn(IsoCharacter);
    commands.spawn(Terrain);
}
