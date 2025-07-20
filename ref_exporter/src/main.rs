use bevy::prelude::*;
use iso::{IsoPlugin, IsoCamera, RefCamera, RefCharacter};
use debug::DebugPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(IsoPlugin);
    app.add_plugins(DebugPlugin);

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn(RefCamera);
    commands.spawn(RefCharacter);
    commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 8.0, 5.0)));
}
