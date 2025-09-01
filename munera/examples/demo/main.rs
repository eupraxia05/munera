use munera::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(MuneraPlugin);
    app.add_systems(Startup, spawn_camera);
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}
