use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_ecs_tilemap::prelude::*;
use bevy::render::camera::RenderTarget;

const MAP_SIZE: u32 = 10;

pub struct IsoPlugin;

impl Plugin for IsoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);

        app.add_systems(Update, handle_added_characters);
        app.add_systems(Update, handle_added_iso_cameras);
        app.add_systems(Update, handle_added_terrains);
        app.add_systems(Update, handle_added_ref_cameras);
        app.add_systems(Update, handle_added_ref_characters);

        app.insert_resource(TerrainTexture::default());

        app.register_type::<IsoCharacter>();
        app.register_type::<IsoCamera>();
        app.register_type::<RefCamera>();
        app.register_type::<Terrain>();
        app.register_type::<TerrainTexture>();
    }
}

#[derive(Component, Default, Reflect)]
pub struct IsoCharacter;

#[derive(Component, Reflect)]
pub struct IsoCamera;

#[derive(Component, Reflect)]
pub struct Terrain {
    pub texture: Handle<Image>
}

#[derive(Component, Reflect, Default)]
pub struct RefCamera {
    pub target: RenderTarget,
}

#[derive(Component, Reflect, Default)]
pub struct RefCharacter {
    pub gltf_path: Option<String>
}

#[derive(Resource, Reflect, Default)]
struct TerrainTexture(Handle<Image>);

fn handle_added_iso_cameras(
    mut commands: Commands,
    mut added_iso_cameras: Query<Entity, Added<IsoCamera>>,
) {
    for cam_ent in added_iso_cameras.iter_mut() {
        commands.entity(cam_ent).insert(Camera2d);
    }
}

fn handle_added_characters(
    mut commands: Commands, 
    mut added_characters: Query<Entity, Added<IsoCharacter>>,
) {
    for char_ent in added_characters.iter_mut() {
        commands.entity(char_ent).insert(Sprite::default());
    }
}

fn handle_added_terrains(
    mut commands: Commands,
    mut added_terrains: Query<(Entity, &Terrain), Added<Terrain>>,
) {
    for (terrain_ent, terrain) in added_terrains.iter_mut() {
        let map_size = TilemapSize { x: MAP_SIZE, y: MAP_SIZE };
        let mut tile_storage = TileStorage::empty(map_size);

        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let tile_pos = TilePos { x, y };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(terrain_ent),
                        ..default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }

        let tile_size = TilemapTileSize { x: 64.0, y: 32.0 };
        let grid_size = tile_size.into();
        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        commands.entity(terrain_ent).insert(TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(terrain.texture.clone_weak()),
            tile_size,
            anchor: TilemapAnchor::Center,
            ..default()
        });
    }
}

fn handle_added_ref_cameras(
    mut commands: Commands,
    added_cameras: Query<(Entity, &RefCamera), Added<RefCamera>>
) {
    for (cam_ent, cam) in added_cameras {
        commands.entity(cam_ent).insert((
            Camera3d::default(),
            Projection::from(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 6.0,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                target: cam.target.clone(),
                ..default()
            }
        ));
    }
}

fn handle_added_ref_characters(
    mut commands: Commands,
    added_characters: Query<(Entity, &RefCharacter), Changed<RefCharacter>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    for (char_ent, character) in added_characters {
        if let Some(gltf_path) = character.gltf_path.clone() {
            let asset_path = GltfAssetLabel::Scene(0).from_asset(gltf_path);
            let scene_asset = asset_server.load(asset_path);
            commands.entity(char_ent).insert(SceneRoot(scene_asset));
        }
    }
}

