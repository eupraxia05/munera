use bevy::prelude::*;
use iso::{IsoPlugin, RefCamera, RefCharacter};
use debug::DebugPlugin;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use bevy_image_export::{ImageExportPlugin, ImageExport, ImageExportSettings, ImageExportSource};
use bevy::render::RenderPlugin;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::camera::RenderTarget;
use bevy_file_dialog::prelude::*;
use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::asset::io::memory::{Dir, MemoryAssetReader};
use std::path::{Path, PathBuf};
use directories::ProjectDirs;

fn main() {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    let mut app = App::new();

    let memory_dir = MemoryDir {
        dir: Dir::default()
    };

    let reader = MemoryAssetReader {
        root: memory_dir.dir.clone()
    };

    app.insert_resource(memory_dir);
    
    app.register_asset_source(AssetSourceId::from_static("memory"),
        AssetSource::build().with_reader(move || Box::new(reader.clone())));
    app.add_plugins(DefaultPlugins
        .set(RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..default()
        })
    );
    app.add_plugins(export_plugin);
    app.add_plugins(IsoPlugin);
    app.add_plugins(DebugPlugin);
    app.add_plugins(FileDialogPlugin::default()
        .with_load_file::<SceneFileDialog>()
        .with_pick_directory::<ExportDirFileDialog>()
    );

    app.add_systems(Startup, startup);
    app.add_systems(EguiPrimaryContextPass, ui);
    app.add_systems(Update, handle_export_button_clicked.after(update_export));
    app.add_systems(Update, update_export);
    app.add_systems(Update, on_scene_file_dialog_loaded);
    app.add_systems(Update, handle_export_dir_file_dialog_picked);

    app.add_event::<ExportButtonClickedEvent>();

    app.run();

    export_threads.finish();
}

#[derive(Resource)]
struct MemoryDir {
    dir: Dir
}

fn startup(
    mut commands: Commands, 
    mut images: ResMut<Assets<Image>>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
) {
    commands.spawn(RefCamera::default());
    commands.spawn(RefCharacter::default());
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
    mut commands: Commands,
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
                ui.horizontal(|ui| {
                    if ui.button("Change Scene").clicked() {
                       commands.dialog().load_file::<SceneFileDialog>(); 
                    } 
                });
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
    export_source: Handle<ImageExportSource>,
    frames_exported: i32,
    final_export_dir: PathBuf,
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
) {
    if !export_button_clicked_event_reader.is_empty() {
        commands.dialog().pick_directory_path::<ExportDirFileDialog>();
        export_button_clicked_event_reader.clear();
    }
}

struct ExportDirFileDialog;

fn handle_export_dir_file_dialog_picked( 
    mut commands: Commands,
    mut export_dir_file_dialog_picked_event_reader: EventReader<DialogDirectoryPicked<ExportDirFileDialog>>,
    mut export_state: ResMut<ExportState>,
) {
    if !export_dir_file_dialog_picked_event_reader.is_empty() {
        let ev = export_dir_file_dialog_picked_event_reader.read().last().unwrap();
        info!("picked: {:?}", ev.path);
        export_state.final_export_dir = ev.path.clone();
        commands.run_system_cached(start_export);
        export_dir_file_dialog_picked_event_reader.clear();
    }

}

fn start_export(
    mut commands: Commands,
    mut export_state: ResMut<ExportState>
) {
    let project_dirs = ProjectDirs::from("", "munera", "ref_exporter").unwrap();
    let export_dir = project_dirs.cache_dir().join("export");

    std::fs::remove_dir_all(export_dir.clone());
    std::fs::create_dir_all(export_dir.clone());

    info!("Exporting to temp dir {:?}", export_dir);

    export_state.export_ent = Some(commands.spawn((
        ImageExport(export_state.export_source.clone()),
        ImageExportSettings {
            output_dir: export_dir.as_os_str().to_string_lossy().into(),
            extension: "png".into()
        }
    )).id());
    export_state.frames_exported = 1;

    info!("Copying to final output dir {:?}", export_state.final_export_dir);

    std::fs::read_dir(export_dir.clone()).unwrap().for_each(|f| { 
        let filename = f.unwrap().file_name().clone();
        std::fs::copy(filename.clone(), export_state.final_export_dir.join(filename)); 
    });
}

fn update_export(
    mut commands: Commands,
    mut export_state: ResMut<ExportState>
) {
    if export_state.is_exporting() {
        if export_state.frames_exported >= 1 {
            commands.entity(export_state.export_ent.unwrap()).despawn();
            export_state.export_ent = None;
        }
        export_state.frames_exported += 1;
    }
}

struct SceneFileDialog;

fn on_scene_file_dialog_loaded(
    mut file_loaded_events: EventReader<DialogFileLoaded<SceneFileDialog>>,
    mut ref_char_query: Query<&mut RefCharacter>,
    asset_server: Res<AssetServer>,
    mem_dir: ResMut<MemoryDir>,
) {
    if let Ok(mut char) = ref_char_query.single_mut() {
        for ev in file_loaded_events.read() {
            let file_name = ev.path.file_name().unwrap().into();
            info!("file name: {:?}", file_name);
            let gltf_handle = mem_dir.dir.insert_asset(Path::new(file_name), ev.contents.clone());
            info!("asset path: {:?} ", format!("memory://{}", file_name.to_str().unwrap()));
            char.gltf_path = Some(format!("memory://{}", file_name.to_str().unwrap()));
        }
    }
} 
