//! Spawn the main level by triggering other observers.

use avian2d::prelude::LinearVelocity;
use bevy::color::palettes::basic::{BLUE, GREEN, RED, YELLOW};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // commands.trigger(SpawnPlayer);

    commands.spawn((
        Name::new("TestBodySun"),
        crate::game::orbital::celestial::CelestialBodyBundle::standard(100.0, 10000000.0, YELLOW)
            .with_transform(Transform::from_xyz(0.0, 0.0, 0.0))
            .with_static(),
        LinearVelocity::from(Vec2::new(-5.0, -5.0)),
    ));

    commands.spawn((
        Name::new("TestBodyA"),
        crate::game::orbital::celestial::CelestialBodyBundle::standard(10.0, 150.0, RED)
            .with_transform(Transform::from_xyz(-200.0, 0.0, 0.0))
            .with_dynamic(),
        LinearVelocity::from(Vec2::new(10.0, 30.0)),
    ));

    commands.spawn((
        Name::new("TestBodyB"),
        crate::game::orbital::celestial::CelestialBodyBundle::standard(50.0, 10000.0, BLUE)
            .with_transform(Transform::from_xyz(400.0, 0.0, 0.0))
            .with_dynamic(),
        LinearVelocity::from(Vec2::new(-100.0, -120.0)),
    ));

    commands.spawn((
        Name::new("TestBodyB"),
        crate::game::orbital::celestial::CelestialBodyBundle::standard(30.0, 1000.0, GREEN)
            .with_transform(Transform::from_xyz(400.0, 400.0, 0.0))
            .with_dynamic(),
        LinearVelocity::from(Vec2::new(-2.0, -50.0)),
    ));
}
