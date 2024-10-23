use bevy::{prelude::*, render::view::RenderLayers};
use std::f32::consts::PI;

pub const TILE_ROTATION: f32 = -PI / 4.;
pub const SCALE: f32 = 4.;
pub const WALL_SLICES: u32 = 12;
pub const WALL_SIZE: u32 = 12;
pub const WALL_STEP_X: u32 = 8;
pub const WALL_STEP_Y: u32 = 8;

pub enum StackDirection {
    WEST,
    NORTH,
    EAST,
    SOUTH,
}

pub fn create_spritestack(
    name: &str,
    x: f32,
    y: f32,
    dir: StackDirection,
    commands: &mut Commands,
    asset_server: &mut Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> Entity {
    let texture: Handle<Image> = asset_server.load(name.to_string() + ".png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(WALL_SIZE), WALL_SLICES, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let id = commands
        .spawn(SpriteStack {
            global_transform: GlobalTransform::default(),
            inherited_visibility: InheritedVisibility::default(),
            transform: Transform::from_xyz(x, y, 0.).with_scale(Vec3::splat(SCALE)),
        })
        .with_children(|parent| {
            let rotation = match dir {
                StackDirection::EAST => PI / 2.,
                StackDirection::WEST => -PI / 2.,
                StackDirection::NORTH => PI,
                StackDirection::SOUTH => 0.,
            };
            for i in 0..WALL_SLICES {
                parent.spawn(SpriteStackSlice {
                    sprite: SpriteBundle {
                        texture: texture.clone(),
                        transform: Transform::from_xyz(0., i as f32, 0.)
                            .with_rotation(Quat::from_rotation_z(TILE_ROTATION + rotation)),
                        ..default()
                    },
                    texture_atlas: TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: i as usize,
                    },
                    layer: RenderLayers::layer(i as usize),
                });
                // HEH, duplicate with a light offset to smooth out edges.
                parent.spawn(SpriteStackSlice {
                    sprite: SpriteBundle {
                        texture: texture.clone(),
                        transform: Transform::from_xyz(0., i as f32 - 0.5, 0.)
                            .with_rotation(Quat::from_rotation_z(TILE_ROTATION + rotation)),
                        ..default()
                    },
                    texture_atlas: TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: i as usize,
                    },
                    layer: RenderLayers::layer(i as usize),
                });
            }
        })
        .id();
    return id;
}

#[derive(Bundle)]
struct SpriteStackSlice {
    sprite: SpriteBundle,
    texture_atlas: TextureAtlas,
    layer: RenderLayers,
}

#[derive(Bundle)]
struct SpriteStack {
    inherited_visibility: InheritedVisibility,
    global_transform: GlobalTransform,
    transform: Transform,
}
