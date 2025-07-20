use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        info!("adding debug plugin");
        app.add_plugins(EguiPlugin::default());
        app.add_plugins(WorldInspectorPlugin::default());
    }
}
