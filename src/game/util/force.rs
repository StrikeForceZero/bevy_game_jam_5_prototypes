pub fn calculate_centrifugal_force(mass: f32, angular_velocity: f32, radius: f32) -> f32 {
    mass * angular_velocity.powi(2) * radius
}
