//! Spawn the main level by triggering other observers.

use avian2d::prelude::LinearVelocity;
use bevy::color::palettes::basic::{BLUE, GREEN, RED, YELLOW};
use bevy::color::palettes::css::ORANGE;
use bevy::prelude::*;

use crate::game::orbital::celestial::CelestialBodyBundle;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // commands.trigger(SpawnPlayer);

    const MASS_SCALE: f32 = 100000.0; // 2500000000000.0;

    #[rustfmt::skip]
    let bodies = [
        ("TestBodySun1", 100.0, 500000.0, YELLOW, Vec2::new(0.0, 0.0), Vec2::new(-15.0, -5.0)),
        ("TestBodySun2", 100.0, 500000.0, ORANGE, Vec2::new(0.0, 400.0), Vec2::new(15.0, 5.0)),
        ("TestBodyA", 15.0, 1500.0, RED, Vec2::new(-400.0, 0.0), Vec2::new(20.0, 15.0)),
        ("TestBodyB", 50.0, 10000.0, BLUE, Vec2::new(400.0, 0.0), Vec2::new(-10.0, -20.0)),
        ("TestBodyB", 30.0, 1000.0, GREEN, Vec2::new(400.0, 400.0), Vec2::new(5.0, -35.0)),
    ];

    for (name, radius, mass, color, pos, vel) in bodies {
        let body = CelestialBodyBundle::standard(radius, mass * MASS_SCALE, color)
            .with_transform(Transform::from_translation(pos.extend(0.0)))
            .with_dynamic();
        commands.spawn((Name::new(name), body, LinearVelocity::from(vel)));
    }
}
