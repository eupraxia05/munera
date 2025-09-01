use bevy::prelude::*;
use bevy::window::{WindowResolution, Window, WindowPlugin};
use debug::DebugPlugin;

pub extern crate bevy;
pub extern crate iso;
pub extern crate debug;

pub mod prelude {
    pub use bevy;
    pub use bevy::prelude::*;
    pub use crate::iso;
    pub use crate::debug;
    pub use crate::MuneraPlugin;
}

pub struct MuneraPlugin;

impl Plugin for MuneraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(640., 480.)
                    .with_scale_factor_override(1.),
                ..default()
            }),
            ..default()
        }))
            .add_plugins(DebugPlugin);
    }
}
