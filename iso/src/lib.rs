use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use std::f32::consts::FRAC_PI_2;
use bevy_ecs_tilemap::prelude::*;

const MAP_SIZE: u32 = 10;

pub struct IsoPlugin;

impl Plugin for IsoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);

        app.add_systems(Startup, startup);
        app.add_systems(Update, handle_added_characters);
        app.add_systems(Update, handle_added_iso_cameras);
        app.add_systems(Update, handle_added_terrains);

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
pub struct Terrain;

#[derive(Component, Reflect)]
pub struct RefCamera;

#[derive(Resource, Reflect, Default)]
struct TerrainTexture(Handle<Image>);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, 
    mut terrain_texture: ResMut<TerrainTexture>,
) {
    terrain_texture.0 = asset_server.load("iso_color.png");
}

fn handle_added_iso_cameras(
    mut commands: Commands,
    mut added_iso_cameras: Query<Entity, Added<IsoCamera>>,
) {
    for mut cam_ent in added_iso_cameras.iter_mut() {
        commands.entity(cam_ent).insert(Camera2d);
    }
}

fn handle_added_characters(
    mut commands: Commands, 
    mut added_characters: Query<Entity, Added<IsoCharacter>>,
    asset_server: Res<AssetServer>,
) {
    for mut char_ent in added_characters.iter_mut() {
        commands.entity(char_ent).insert((IsoCharacter, Sprite::default()));
    }
}

fn handle_added_terrains(
    mut commands: Commands,
    mut added_terrains: Query<Entity, Added<Terrain>>,
    mut terrain_texture: Res<TerrainTexture>,
) {
    for mut terrain_ent in added_terrains.iter_mut() {
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
            texture: TilemapTexture::Single(terrain_texture.0.clone_weak()),
            tile_size,
            anchor: TilemapAnchor::Center,
            ..default()
        });
    }
}
