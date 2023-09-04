use bevy::window::PrimaryWindow;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct AsteriodSpawner(Timer);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Timeout(f32);

#[derive(Component)]
struct PlayerID(u32);

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Asteriod;

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
                move_camera_with_player,
                spawn_asteriods,
                handle_asteriod_bullet_collision,
                update_score
            ))
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
    asset_server: Res<AssetServer>,
) {

    let text_style = TextStyle { font: asset_server.load("fonts/FiraMono-Medium.ttf"), font_size:32.0, color :Color::RED};

    commands.spawn(
        SpriteBundle{
            sprite: Sprite{
                custom_size: Some(Vec2::new(32., 32.)),
                ..default()
            },
            texture: asset_server.load("spaceship.png"),
            ..default()
        }).insert((Velocity(Vec2 { x: 0., y: 0. }), PlayerID(0), Health(100.)));

        commands.spawn(Camera2dBundle::default());

        commands.spawn(Text2dBundle{
                text: Text { sections: vec![TextSection::new("Score: ", text_style.clone()), TextSection::new("", text_style.clone())], ..default()},
                ..default()
            });

        commands.insert_resource(AsteriodSpawner(Timer::from_seconds(2.0, TimerMode::Repeating)));
        commands.insert_resource(Score(0));
}


fn move_camera_with_player(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<PlayerID>)>,
    player: Query<& Transform, (With<PlayerID>, Without<Camera2d>)>
){
    camera.single_mut().translation = player.single().translation;
}

fn update_score(
    score: Res<Score>,
    mut scoreboard_query: Query<&mut Text>
){
    scoreboard_query.single_mut().sections[1].value = score.0.to_string();
}

fn handle_asteriod_bullet_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    bullet_query: Query<(Entity, &Transform), (With<Bullet>, Without<Asteriod>)>,
    mut asteriod_query: Query<(Entity, &Transform, &mut Health), (With<Asteriod>, Without<Bullet>)>
){
    for(asteriod, asteriod_transform,mut asteriod_health) in &mut asteriod_query{
        for (bullet, bullet_transform) in &bullet_query{
            if bullet_transform.translation.distance(asteriod_transform.translation) < 50.0{
                asteriod_health.0-= 10.0;
                if asteriod_health.0<=0.0{
                    score.0+=1;
                    commands.entity(asteriod).despawn();
                }
                commands.entity(bullet).despawn();
            }
        }
    }
}


fn spawn_asteriods(
    mut commands: Commands,
    time: Res<Time>,
    mut asteriod_spawner: ResMut<AsteriodSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<&Transform, With<PlayerID>>,
    window_query: Query<&Window, With<PrimaryWindow>>
){
    asteriod_spawner.0.tick(time.delta());

    if asteriod_spawner.0.finished(){
        // spawn asteriod on the edges of the visible screen
        let dim = &window_query.single().resolution;
        let player_pos = player_query.single().translation;
        // find all possible spawn areas
        let spawn_areas = vec![
            (
                player_pos.x-dim.width()/2.0 -50.0 ..  player_pos.x-dim.width()/2.0 -20.0,
                player_pos.y-dim.height()/2.0 .. player_pos.y+dim.height()/2.0
            ),
            (
                player_pos.x+dim.width()/2.0 +20.0 ..  player_pos.x+dim.width()/2.0 + 50.0, 
                player_pos.y-dim.height()/2.0 .. player_pos.y+dim.height()/2.0
            ),
            (
                player_pos.x-dim.width()/2.0 .. player_pos.x+dim.width()/2.0,
                player_pos.y+dim.height()/2.0 +20.0 ..  player_pos.y+dim.height()/2.0 + 50.0
            ),
            (
                player_pos.x-dim.width()/2.0 .. player_pos.x+dim.width()/2.0,
                player_pos.y-dim.height()/2.0 -50.0 ..  player_pos.y-dim.height()/2.0 -20.0
            )];
        // Pick one area for the asteriod to spawn
        let (x, y) = spawn_areas.choose(&mut rand::thread_rng()).unwrap();
        // set the translation of the asteriod
        let translation = Vec3::new(
            rand::thread_rng().gen_range(x.clone()),
            rand::thread_rng().gen_range(y.clone()),
            0.0
        );
        // Spawn the new asteriod with the correct rotation to move towards the player
        commands.spawn(
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(50.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform{
                    translation,
                    rotation: Quat::from_rotation_arc(Vec3::Y, (player_pos-translation).normalize()),
                    ..default()
                },
                ..default()
            }).insert(Velocity(Vec2::new(0., 100.)))
            .insert(Timeout(20.0))
            .insert(Health(50.0))
            .insert(Asteriod);
    }
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

    // Games typically only have one window (the primary window)
    if let Some(position) = q_windows.single().cursor_position() {
        let mut cloned = position;
        cloned.x = cloned.x - q_windows.single().resolution.width() / 2.;
        cloned.y = q_windows.single().resolution.height() / 2. - cloned.y;

        let to_player = cloned.normalize();

        // get the quaternion to rotate the player to face the cursor
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
fn move_player(
    input: Res<Input<KeyCode>>, 
    mut query: Query<&mut Velocity, With<PlayerID>>
) {
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
fn update_transforms(
    time: Res<Time>, 
    mut moving_object: Query<(&Velocity, &mut Transform)>
) {
    for (vel, mut transform) in &mut moving_object {
        let mut vec = vel.0.clone().extend(0.);
        vec = transform.rotation.mul_vec3(vec);
        transform.translation += vec * time.delta_seconds();
    }
}

