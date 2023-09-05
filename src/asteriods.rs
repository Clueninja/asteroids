use bevy::prelude::*;
use rand::prelude::*;

use crate::{Asteriod, Health, Lifetime, Velocity};

#[derive(Component)]
struct Spinning {
    speed: f32,
}

#[derive(Bundle)]
pub struct AsteriodBundle {
    _asteriod: Asteriod,
    health: Health,
    spinning: Spinning,
    velocity: Velocity,
    lifetime: Lifetime,
    #[bundle()]
    sprite_bundle: SpriteBundle,
}

impl AsteriodBundle {
    pub fn new(asset_server: ResMut<AssetServer>, position: Vec3) -> Self {
        Self {
            _asteriod: Asteriod,
            health: Health(100.0),
            spinning: Spinning { speed: 0.0 },
            velocity: Velocity(Vec2::new(0.0, 0.0)),
            lifetime: Lifetime(20.0),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..default()
                },
                texture: asset_server.load("asteriod.png"),
                transform: Transform::from_translation(position),
                ..default()
            },
        }
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
            velocity: Velocity(Vec2 { x: rand::thread_rng().gen(), y: rand::thread_rng().gen() }.normalize() * 100.0),
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
