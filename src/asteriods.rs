use bevy::prelude::*;
use rand::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{Health, Lifetime, Velocity, Player, Score};

// lower the more difficult
static ASTERIOD_DIFFICUTY:f32 = 20.0;

#[derive(Resource)]
pub struct AsteriodSpawner{
    pub timer: Timer,
    pub current_duration: f32
}

#[derive(Component)]
pub struct Asteriod;


#[derive(Component)]
pub struct Spinning {
    pub speed: f32,
}

#[derive(Bundle)]
pub struct AsteriodBundle {
    pub _asteriod: Asteriod,
    pub health: Health,
    pub spinning: Spinning,
    pub velocity: Velocity,
    pub lifetime: Lifetime,
    #[bundle()]
    pub sprite_bundle: SpriteBundle,
}

impl AsteriodBundle {
    pub fn new(score: Res<Score>, asset_server: ResMut<AssetServer>, position: Vec3, velocity: Vec2) -> Self {
        // ASTEROID_DIFFICULTY = 20.0
        // Score |  Min Health  |  Max Health
        //   0   |     50       |     250
        //  10   |     75       |     325
        //  20   |    100       |     400

        let health = thread_rng().gen::<f32>()*100.0 * ((score.0 as f32)/ASTERIOD_DIFFICUTY + 2.0) + 50.0 * ((score.0 as f32)/ASTERIOD_DIFFICUTY + 1.0);

        Self {
            _asteriod: Asteriod,
            health: Health(health),
            spinning: Spinning { speed: 0.0 },
            velocity: Velocity(velocity),
            lifetime: Lifetime(20.0),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    // increase size a little so it is never a single pixel
                    custom_size: Some(Vec2::new(health+20.0, health+20.0)),
                    ..default()
                },
                texture: asset_server.load("asteriod.png"),
                transform: Transform::from_translation(position),
                ..default()
            },
        }
    }
}


pub fn shrink_asteriod(
    health: &Health,
    sprite: &mut Sprite,
){
    sprite.custom_size = Some(Vec2 { x: health.0 + 20.0, y: health.0 + 20.0 });
}


pub fn spawn_asteriods(
    mut commands: Commands,
    time: Res<Time>,
    score: Res<Score>,
    mut asteriod_spawner: ResMut<AsteriodSpawner>,
    asset_server: ResMut<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>
){
    asteriod_spawner.timer.tick(time.delta());

    if asteriod_spawner.timer.finished(){
        // spawn asteriod on the edges of the visible screen
        let dim = &window_query.single().resolution;
        let player_pos = player_query.single().translation;
        // find all possible spawn areas
        let spawn_areas = vec![
            (
                player_pos.x-dim.width()/2.0 -200.0 ..  player_pos.x-dim.width()/2.0 -100.0,
                player_pos.y-dim.height()/2.0 .. player_pos.y+dim.height()/2.0
            ),
            (
                player_pos.x+dim.width()/2.0 +100.0 ..  player_pos.x+dim.width()/2.0 + 200.0, 
                player_pos.y-dim.height()/2.0 .. player_pos.y+dim.height()/2.0
            ),
            (
                player_pos.x-dim.width()/2.0 .. player_pos.x+dim.width()/2.0,
                player_pos.y+dim.height()/2.0 +100.0 ..  player_pos.y+dim.height()/2.0 + 200.0
            ),
            (
                player_pos.x-dim.width()/2.0 .. player_pos.x+dim.width()/2.0,
                player_pos.y-dim.height()/2.0 -200.0 ..  player_pos.y-dim.height()/2.0 -100.0
            )];
        // Pick one area for the asteriod to spawn
        let (x, y) = spawn_areas.choose(&mut rand::thread_rng()).unwrap();
        // set the translation of the asteriod
        let translation = Vec3::new(
            rand::thread_rng().gen_range(x.clone()),
            rand::thread_rng().gen_range(y.clone()),
            0.0
        );
        let vel = player_pos-translation;
        let velocity = Vec2::new(vel.x, vel.y).normalize() * 50.0 * ((score.0 as f32)/ASTERIOD_DIFFICUTY + 1.0);
        // Spawn the new asteriod with the correct rotation to move towards the player
        commands.spawn(AsteriodBundle::new(score, asset_server, translation, velocity));
    }
}


#[derive(Component)]
pub struct Particle;

#[derive(Component)]
pub struct FadeOut {
    timer: Timer,
}

pub fn fadeout_sprites(
    mut commands: Commands,
    time: Res<Time>,
    mut sprite_query: Query<(Entity, &mut FadeOut, &mut Sprite)>,
) {
    for (entity, mut fadeout, mut sprite) in &mut sprite_query {
        fadeout.timer.tick(time.delta());
        if fadeout.timer.finished() {
            commands.entity(entity).despawn();
        } else {
            sprite.color.set_a(fadeout.timer.percent_left());
        }
    }
}

#[derive(Bundle)]
pub struct ParticleBundle {
    _particle: Particle,
    spinning: Spinning,
    velocity: Velocity,
    fadeout: FadeOut,
    #[bundle()]
    sprite_bundle: SpriteBundle,
}

impl ParticleBundle {
    pub fn new(asset_server: &mut ResMut<AssetServer>, translation: Vec3) -> Self {
        Self {
            _particle: Particle,
            spinning: Spinning { speed: 0.0 },
            velocity: Velocity(Vec2 { x: rand::thread_rng().gen::<f32>() -0.5, y: rand::thread_rng().gen::<f32>()-0.5 }.normalize() * 100.0),
            fadeout: FadeOut {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2 { x: 10.0, y: 10.0 }),
                    ..default()
                },
                transform: Transform::from_translation(translation),
                texture: asset_server.load("asteriod.png"),
                ..default()
            },
        }
    }
}
