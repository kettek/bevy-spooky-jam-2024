use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::view::{Layer, RenderLayers},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (sprite_movement/*, check_stacks*/))
        .run();
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
const SCALE: f32 = 2.;
const WALL_SLICES: u32 = 12;
const WALL_SIZE: u32 = 12;
const WALL_STEP_X: u32 = 8;
const WALL_STEP_Y: u32 = 8;

fn create_spritestack(
    name: &str,
    x: f32,
    y: f32,
    commands: &mut Commands,
    asset_server: &mut Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
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
            for i in 0..WALL_SLICES {
                parent.spawn(SpriteStackSlice {
                    sprite: SpriteBundle {
                        texture: texture.clone(),
                        transform: Transform::from_xyz(0., i as f32, 0.)
                            .with_rotation(Quat::from_rotation_z(TILE_ROTATION)),
                        ..default()
                    },
                    texture_atlas: TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: i as usize,
                    },
                    layer: RenderLayers::layer(i as usize),
                });
            }
        });
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
struct Tile {
    kind: String,
    stack: SpriteStack,
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

    //commands.spawn((Camera2dBundle { ..default() }, render_layers.clone()));
    for x in (0..8).rev() {
        for y in (0..8).rev() {
            if x > 0 && x < 7 && y > 0 && y < 7 {
                continue;
            }
            // Calculate our X and Y as oblique.
            let mut real_x = (y - x) as f32 * WALL_STEP_X as f32 * SCALE;
            let mut real_y = (y + x) as f32 * WALL_STEP_Y as f32 * SCALE;

            create_spritestack(
                "wall",
                real_x,
                real_y,
                &mut commands,
                &mut asset_server,
                &mut texture_atlas_layouts,
            );
        }
    }

    create_spritestack(
        "wall",
        100. + WALL_STEP_X as f32 * SCALE * 2.,
        WALL_SIZE as f32 * SCALE - WALL_STEP_Y as f32 * SCALE * 2.,
        &mut commands,
        &mut asset_server,
        &mut texture_atlas_layouts,
    );
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
