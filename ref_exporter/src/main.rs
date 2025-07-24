use bevy::prelude::*;
use iso::{IsoPlugin, RefCamera, RefCharacter};
use debug::DebugPlugin;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use bevy_image_export::{ImageExportPlugin, ImageExport, ImageExportSettings, ImageExportSource};
use bevy::render::RenderPlugin;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::camera::RenderTarget;

fn main() {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    let mut app = App::new();

    app.add_plugins(DefaultPlugins
        .set(RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..default()
        })
    );
    app.add_plugins(export_plugin);
    app.add_plugins(IsoPlugin);
    app.add_plugins(DebugPlugin);

    app.add_systems(Startup, startup);
    app.add_systems(EguiPrimaryContextPass, ui);
    app.add_systems(Update, handle_export_button_clicked);

    app.add_event::<ExportButtonClickedEvent>();

    app.run();

    export_threads.finish();
}

fn startup(
    mut commands: Commands, 
    mut images: ResMut<Assets<Image>>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
) {
    commands.spawn(RefCamera::default());
    commands.spawn(RefCharacter);
    commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 8.0, 5.0)));
    
    let texture_size = Extent3d {
        width: 256,
        height: 256,
        ..default()
    };

    let mut export_texture = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: texture_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    export_texture.resize(texture_size);

    let export_texture_handle = images.add(export_texture);

    commands.spawn(RefCamera {
        target: RenderTarget::Image(export_texture_handle.clone().into()),
        ..default()
    });
    
    commands.insert_resource(ExportState {
        export_source: export_sources.add(export_texture_handle),
        ..default()
    });
}

fn ui(
    mut egui_contexts: EguiContexts, 
    export_state: Res<ExportState>, 
    mut export_button_clicked_event_writer: EventWriter<ExportButtonClickedEvent>
) {
    if let Ok(ctx) = egui_contexts.ctx_mut() {
        egui::Window::new("Ref Export").show(ctx, |ui| {
            if export_state.is_exporting() {
                ui.label("Exporting...");
            }
            else {
                if ui.button("Export").clicked() {
                    export_button_clicked_event_writer.write_default();                 
                }
            }
        });
    }
}

#[derive(Resource, Default)]
struct ExportState {
    export_ent: Option<Entity>,
    export_source: Handle<ImageExportSource>
}

impl ExportState {
    fn is_exporting(&self) -> bool {
        self.export_ent.is_some()
    }
}

#[derive(Event, Default)]
struct ExportButtonClickedEvent;

fn handle_export_button_clicked(
    mut commands: Commands,
    mut export_button_clicked_event_reader: EventReader<ExportButtonClickedEvent>,
    export_state: Res<ExportState>
) { 
    if !export_button_clicked_event_reader.is_empty() {
        export_button_clicked_event_reader.clear();
        commands.spawn((
            ImageExport(export_state.export_source.clone()),
            ImageExportSettings {
                output_dir: "out".into(),
                extension: "png".into()
            }
        ));
    }
}
