pub mod asteriods;

use asteriods::{Asteriod, AsteriodSpawner, ParticleBundle, fadeout_sprites, spawn_asteriods};
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy::utils::Duration;


#[derive(Resource)]
struct Score(u32);
    
#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Component)]
pub struct Health(f32);

#[derive(Component)]
pub struct Lifetime(f32);

#[derive(Component)]
pub struct PlayerID(u32);

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct Missile;


#[derive(Resource)]
pub struct PlayerWeaponry{
    pub missile_timer: Timer,
    pub gun_timer: Timer,
}

#[derive(Component)]
pub struct DamageAsteriods{
    pub damage: f32
}


#[derive(Component)]
pub struct SafeZone;



fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (set_crosshair, setup))
        .add_systems(
            Update,
            (
                update_transforms,
                cursor_position,
                move_player,
                fire_weaponry,
                update_timeout,
                move_camera_with_player,
                spawn_asteriods,
                handle_asteriod_bullet_collision,
                update_score,
                check_player_in_safezone,
                fadeout_sprites
            ))
        .run()
}


// Spawn all Normal Entities on Startup
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {

    let text_style = TextStyle { font: asset_server.load("fonts/FiraMono-Medium.ttf"), font_size:32.0, color :Color::RED};

    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(2000.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::GRAY)),
            transform: Transform::from_xyz(0.0, 0.0, -20.0),
            ..default()
        }).insert(SafeZone);

    commands.spawn(
        SpriteBundle{
            sprite: Sprite{
                custom_size: Some(Vec2::new(48., 48.)),
                ..default()
            },
            texture: asset_server.load("spaceship-fotor-bg-remover-20230906141855.png"),
            ..default()
        }).insert((Velocity(Vec2 { x: 0., y: 0. }), PlayerID(0), Health(100.)));


        commands.spawn(
            Text2dBundle{
                text: Text { 
                    sections: vec![TextSection::new("Score: ", text_style.clone()), TextSection::new("", text_style.clone())], 
                    ..default()
                },
                ..default()
        });

        commands.spawn(Camera2dBundle::default());


        commands.insert_resource(AsteriodSpawner{timer: Timer::from_seconds(2.0, TimerMode::Repeating), current_duration: 2.0});
        commands.insert_resource(
            PlayerWeaponry{
                missile_timer: Timer::from_seconds(2.0, TimerMode::Once),
                gun_timer: Timer::from_seconds(0.2, TimerMode::Once),
            }
        );
        commands.insert_resource(Score(0));
}

fn set_crosshair(
    mut window: Query<&mut Window, With<PrimaryWindow>>
){
    window.single_mut().cursor.icon = CursorIcon::Crosshair;
}

fn move_camera_with_player(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<PlayerID>)>,
    player: Query<& Transform, (With<PlayerID>, Without<Camera2d>)>
){
    camera.single_mut().translation = player.single().translation;
}

fn check_player_in_safezone(
    mut player_query: Query<(&Transform, &mut Health), (With<PlayerID>, Without<SafeZone>)>,
    safezone_query: Query<&Transform, (With<SafeZone>, Without<PlayerID>)>,
    mut asteriod_spawner: ResMut<AsteriodSpawner>
){
    let (player_transform, _player_health ) = player_query.single_mut();
    if player_transform.translation.distance(safezone_query.single().translation) > 2000.0{
        asteriod_spawner.current_duration = 0.5;
        asteriod_spawner.timer.set_duration(Duration::from_secs_f32(0.5));
    }
    else if asteriod_spawner.current_duration != 2.0{
        asteriod_spawner.current_duration = 2.0;
        asteriod_spawner.timer.set_duration(Duration::from_secs_f32(2.0));
    }
}


fn update_score(
    score: Res<Score>,
    mut scoreboard_query: Query<&mut Text>
){
    scoreboard_query.single_mut().sections[1].value = score.0.to_string();
}

// Fix bug
fn handle_asteriod_bullet_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut asset_server: ResMut<AssetServer>,
    missile_query: Query<(Entity, &Transform, &DamageAsteriods), (With<Missile>, Without<Asteriod>)>,
    bullet_query: Query<(Entity, &Transform, &DamageAsteriods), (Without<Missile>, Without<Asteriod>)>,
    mut asteriod_query: Query<(Entity, &Transform, &mut Health, &mut Sprite), With<Asteriod>>
){
    for(asteriod, asteriod_transform,mut asteriod_health, mut asteriod_sprite) in &mut asteriod_query{
        for (bullet, bullet_transform, damage) in &bullet_query{
            if bullet_transform.translation.distance(asteriod_transform.translation) < asteriod_health.0/2.0 + 10.0{
                asteriod_health.0-= damage.damage;
                asteriod_sprite.custom_size = Some(Vec2 { x: asteriod_health.0, y: asteriod_health.0 });

                commands.spawn(ParticleBundle::new(&mut asset_server, bullet_transform.translation.clone()));

                if asteriod_health.0<=0.0{
                    score.0+=1;
                    commands.entity(asteriod).despawn();
                }
                commands.entity(bullet).despawn();
            }
        }
        for (missile, missile_transform, damage) in &missile_query{
            if missile_transform.translation.distance(asteriod_transform.translation) < asteriod_health.0/2.0 + 10.0{
                if asteriod_health.0 < 2.0 * damage.damage{
                    commands.entity(asteriod).despawn();
                }
                else{
                    asteriod_health.0-= damage.damage;
                    asteriod_sprite.custom_size = Some(Vec2 { x: asteriod_health.0, y: asteriod_health.0 });

                    commands.spawn(ParticleBundle::new(&mut asset_server, missile_transform.translation.clone()));

                    if asteriod_health.0<=0.0{
                        score.0+=1;
                        commands.entity(asteriod).despawn();
                    }
                }
                commands.entity(missile).despawn();
            }
        }
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
fn fire_weaponry(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player_weaponry: ResMut<PlayerWeaponry>,
    query: Query<&Transform, With<PlayerID>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    player_weaponry.missile_timer.tick(time.delta());
    player_weaponry.gun_timer.tick(time.delta());

    if mouse_button_input.pressed(MouseButton::Left) {
        if player_weaponry.gun_timer.finished(){
            commands
                .spawn(MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(5.).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
                    transform: query.single().clone(),
                    ..default()
                })
                .insert(Velocity(Vec2 { x: 0., y: 1000. }))
                .insert(DamageAsteriods{damage: 20.0})
                .insert(Lifetime(1.0));
            player_weaponry.gun_timer.reset();
        }
    }
    if keyboard_input.just_pressed(KeyCode::Space){
        if player_weaponry.missile_timer.finished(){
            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                transform: query.single().clone(),
                ..default()
            }).insert(Velocity(Vec2{x: 0.0, y: 500.0}))
            .insert(DamageAsteriods{damage: 100.0})
            .insert(Lifetime(4.0))
            .insert(Missile);
            player_weaponry.missile_timer.reset();
        }
        else{
            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(5.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::RED)),
                transform: query.single().clone(),
                ..default()
            }).insert(Velocity(Vec2{x: 0.0, y: 500.0}))
            .insert(Lifetime(0.2));
        }
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
    mut query: Query<(Entity, &mut Lifetime)>,
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

