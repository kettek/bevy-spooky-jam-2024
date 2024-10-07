#[derive(Component)]
struct Player;

pub fn create_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        SpriteBundle {
            material: asset_server.load("blep.png").into(),
            transform: Transform::from_xyz(100., 0., 0.),
            ..Default::default()
        },
    ));
}
