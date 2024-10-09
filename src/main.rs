use std::f32::consts::PI;

use bevy::{
    log::tracing_subscriber::field::debug,
    prelude::*,
    render::view::{Layer, RenderLayers},
};
use bevy_common_assets::yaml::YamlAssetPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            YamlAssetPlugin::<Level>::new(&["level.yaml"]),
            YamlAssetPlugin::<KickStart>::new(&["kickstart.yaml"]),
        ))
        .insert_resource(Msaa::Off)
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                sprite_movement, /*, check_stacks*/
                kickstart.run_if(in_state(AppState::Loading)),
                spawn_level.run_if(in_state(AppState::Kickstarted)),
            ),
        )
        .run();
}

#[derive(serde::Deserialize, Asset, TypePath)]
struct Level {
    name: String,
    tiles: std::collections::HashMap<char, String>,
    map: Vec<String>,
}

#[derive(Resource)]
struct LevelHandle(Handle<Level>);

#[derive(serde::Deserialize, Asset, TypePath)]
struct KickStart {
    first_level: String,
    levels: Vec<String>,
}

#[derive(Resource)]
struct KickstartHandle(Handle<KickStart>);

fn kickstart(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut kickstart: ResMut<Assets<KickStart>>,
    mut state: ResMut<NextState<AppState>>,
) {
    kickstart.iter().for_each(|(handle, kickstart)| {
        info!("Kickstarting with first level: {}", kickstart.first_level);
        state.set(AppState::Kickstarted);

        let mut level_name = String::new();
        level_name.push_str("levels/");
        level_name.push_str(&kickstart.first_level);
        level_name.push_str(".level.yaml");

        let level_handle = asset_server.load(level_name);
        commands.insert_resource(LevelHandle(level_handle));

        return;
    });
}

fn spawn_level(
    mut commands: Commands,
    level_handle: Res<LevelHandle>,
    mut levels: ResMut<Assets<Level>>,
    mut state: ResMut<NextState<AppState>>,
    mut asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    info!("Spawning level");

    levels.iter().for_each(|(handle, level)| {
        info!("Level: {}", level.name);
        let max_y = level.map.len() as f32;
        for (y, row) in level.map.iter().enumerate() {
            let max_x = row.len() as f32;
            for (x, tile) in row.chars().enumerate() {
                let mut rel_x = (y as f32 - x as f32) * WALL_STEP_X as f32 * SCALE;
                let mut rel_y = (y as f32 + x as f32) * WALL_STEP_Y as f32 * SCALE;

                let real_x = max_x - rel_x;
                let real_y = max_y - rel_y;

                if level.tiles.contains_key(&tile) {
                    let name = level.tiles.get(&tile).unwrap();
                    let id = create_spritestack(
                        name,
                        0.,
                        0.,
                        StackDirection::SOUTH,
                        &mut commands,
                        &mut asset_server,
                        &mut texture_atlas_layouts,
                    );
                    let tile = commands
                        .spawn((Tile {
                            kind: Kind(name.to_string()),
                            inherited_visibility: InheritedVisibility::default(),
                            global_transform: GlobalTransform::default(),
                            transform: Transform::from_xyz(real_x, real_y, 0.),
                        },))
                        .id();
                    commands.entity(tile).push_children(&[id]);
                    continue;
                }
            }
        }
    });

    state.set(AppState::Level);
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Kickstarted,
    Level,
}

#[derive(Component)]
enum Direction {
    Up,
    Down,
}

#[derive(Component)]
struct TurnTaker {
    max_actions: u32,
    rem_actions: u32,
}

#[derive(Component)]
struct Leveler {
    cur_level: u32,
    cur_xp: u32,
    next_level_xp: u32,
}

#[derive(Component)]
struct Health {
    max_hp: u32,
    cur_hp: u32,
}

#[derive(Component)]
struct Skills {
    pending_points: u32,
    warp: u32,
    heal: u32,
    dematerialize: u32,
    reform: u32,
}

const TILE_ROTATION: f32 = -PI / 4.;
const SCALE: f32 = 4.;
const WALL_SLICES: u32 = 12;
const WALL_SIZE: u32 = 12;
const WALL_STEP_X: u32 = 8;
const WALL_STEP_Y: u32 = 8;

enum StackDirection {
    WEST,
    NORTH,
    EAST,
    SOUTH,
}

fn create_spritestack(
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

#[derive(Component)]
struct Kind(String);

#[derive(Bundle)]
struct Tile {
    kind: Kind,
    inherited_visibility: InheritedVisibility,
    global_transform: GlobalTransform,
    transform: Transform,
}

#[derive(Bundle)]
struct PlayerBundle {
    turn_taker: TurnTaker,
    leveler: Leveler,
    health: Health,
    skills: Skills,
}

fn level_up(mut query: Query<(&mut Leveler, &mut Health, &mut Skills)>) {
    for (mut leveler, mut health, mut skills) in query.iter_mut() {
        if leveler.cur_xp >= leveler.next_level_xp {
            leveler.cur_level += 1;
            leveler.next_level_xp = leveler.cur_level * 1000;
            health.max_hp += 10;
            health.cur_hp = health.max_hp;
            skills.pending_points += 1;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let handle = asset_server.load("kickstart.yaml");
    commands.insert_resource(KickstartHandle(handle));

    let mut render_layers = RenderLayers::layer(0);
    // This camera crap really isn't doing anything
    for i in 0..WALL_SLICES {
        let camera_order = WALL_SLICES as isize - i as isize;
        render_layers = render_layers.with(i as usize);
        // Create camera as well...
        commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    order: camera_order,
                    ..default()
                },
                ..default()
            },
            render_layers.clone(),
        ));
    }
}

/*fn check_stacks(mut query: Query<&mut SpriteStack>) {
    for mut stack in query.iter_mut() {
        //info!("Checking stack");
        /*for slice in &mut stack.slices {
            slice.transform.translation.y += 1.;
            info!("Slice y: {}", slice.transform.translation.y);
        }*/
    }
}*/

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
        }

        if transform.translation.y > 200. {
            *logo = Direction::Down;
        } else if transform.translation.y < -200. {
            *logo = Direction::Up;
        }
    }
}

#[derive(Resource)]
struct AnimationState {
    min: f32,
    max: f32,
    speed: f32,
    current: f32,
}

fn animate(mut sprites: Query<&mut Sprite>, mut state: ResMut<AnimationState>, time: Res<Time>) {}
