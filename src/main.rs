use bevy::window::PrimaryWindow;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct PlayerID(u32);

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Timeout(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_window, setup))
        .add_systems(
            Update,
            (
                update_transforms,
                cursor_position,
                move_player,
                fire_bullet,
                update_timeout,
                move_camera_with_player
            ),
        )
        .run()
}

fn setup_window(
    mut win: Query<&mut Window, With<PrimaryWindow>>,
){
    win.single_mut().title = String::from("Asteriods");
    win.single_mut().cursor.icon = CursorIcon::Crosshair;
}

// Spawn all Normal Entities on Startup
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {

    // Circle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
            ..default()
        },
        Velocity(Vec2 { x: 10., y: 0. }),
    ));

    // Rectangle
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(50.0, 100.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
        ..default()
    });

    // Quad
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(50., 100.)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
            transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
            ..default()
        },
        Velocity(Vec2 { x: 20., y: -10. }),
    ));

    // Hexagon
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
        transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
        ..default()
    });
    let image: Handle<Image> = asset_server.load("spaceship.png");
    commands.spawn(
        SpriteBundle{
            sprite: Sprite{
                custom_size: Some(Vec2::new(32., 32.)),
                ..default()
            },
            texture: image,
            ..default()
        }).insert((Velocity(Vec2 { x: 0., y: 0. }), PlayerID(0), Health(100.)));

        commands.spawn(Camera2dBundle::default());
}


fn move_camera_with_player(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<PlayerID>)>,
    player: Query<& Transform, (With<PlayerID>, Without<Camera2d>)>
){
    camera.single_mut().translation = player.single().translation;
}


// Converts the Cursor position to screen coordinates, then rotates the player to the Cursor
// could make a seperate Component for objects that always rotate to the cursor
// Contains multiple bugs I'm sure
fn cursor_position(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut player_query: Query<&mut Transform, With<PlayerID>>,
) {
    let mut player_transform = player_query.single_mut();
    // get the player translation in 2D
    //let player_translation = player_transform.translation.xy();

    // Games typically only have one window (the primary window)
    if let Some(position) = q_windows.single().cursor_position() {
        let mut cloned = position;
        cloned.x = cloned.x - q_windows.single().resolution.width() / 2.;
        cloned.y = q_windows.single().resolution.height() / 2. - cloned.y;

        let to_player = (cloned).normalize();

        // get the quaternion to rotate from the initial enemy facing direction to the direction
        // facing the player
        let rotate_to_player = Quat::from_rotation_arc(Vec3::Y, to_player.extend(0.));
        player_transform.rotation = rotate_to_player;
    }
}

// If spacebar is pressed, spawn a new Entity, with Bullet, and timeout components with a circle sprite
fn fire_bullet(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    query: Query<&Transform, With<PlayerID>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if input.just_pressed(MouseButton::Left) {
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(5.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
                transform: query.single().clone(),
                ..default()
            })
            .insert(Velocity(Vec2 { x: 0., y: 1000. }))
            .insert(Bullet)
            .insert(Timeout(0.5));
    }
}

// increase and decrease player speed
fn move_player(input: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<PlayerID>>) {
    if input.pressed(KeyCode::W) {
        if query.single().0.y < 200.{
            query.single_mut().0.y += 10.;
        }
    }
    if input.pressed(KeyCode::S) {
        if query.single().0.y >0.{
            query.single_mut().0.y -= 10.;
        }
    }
    if input.pressed(KeyCode::W) && input.pressed(KeyCode::S) {
        query.single_mut().0.y = 0.;
    }
}

// automatically update all entities that have the Timeout Component
fn update_timeout(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Timeout)>,
) {
    for (e, mut timeout) in &mut query {
        timeout.0 -= time.delta_seconds();
        if timeout.0 < 0. {
            commands.entity(e).despawn();
        }
    }
}

// move transforms for entities with a Velocity Component
fn update_transforms(time: Res<Time>, mut moving_object: Query<(&Velocity, &mut Transform)>) {
    for (vel, mut transform) in &mut moving_object {
        let mut vec = vel.0.clone().extend(0.);
        vec = transform.rotation.mul_vec3(vec);
        transform.translation += vec * time.delta_seconds();
    }
}
