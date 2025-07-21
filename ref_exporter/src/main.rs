use bevy::prelude::*;
use iso::{IsoPlugin, IsoCamera, RefCamera, RefCharacter};
use debug::DebugPlugin;
use bevy_egui::{egui, EguiContexts};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(IsoPlugin);
    app.add_plugins(DebugPlugin);

    app.add_systems(Startup, startup);
    app.add_systems(Update, ui);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn(RefCamera);
    commands.spawn(RefCharacter);
    commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 8.0, 5.0)));
}

fn ui(mut egui_contexts: EguiContexts) {
    if let Ok(ctx) = egui_contexts.ctx_mut() {
        egui::Window::new("Title").show(egui_contexts.ctx_mut().unwrap(), |ui| {
            ui.label("hello!");
        });
    }
}
