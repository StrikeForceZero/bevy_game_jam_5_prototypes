use std::f32::consts::PI;

use bevy::math::Vec2;

pub fn create_hollow_circle_vertices(
    inner_radius: f32,
    outer_radius: f32,
    circle_resolution: usize,
) -> Vec<Vec2> {
    let mut vertices = Vec::new();

    let calc_point = |radius: f32, index: usize| -> Vec2 {
        if index > circle_resolution {
            panic!("invalid index got: {index}, expected: 0..={circle_resolution}");
        }
        let theta = 2.0 * PI * index as f32 / circle_resolution as f32;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        Vec2::new(x, y)
    };

    // Generate inside circle vertices
    for i in 0..=circle_resolution {
        vertices.push(calc_point(inner_radius, i));
    }

    // Generate outside circle vertices
    // Going in reverse will ensure the inner start and stop match the outers
    for i in (0..=circle_resolution).rev() {
        vertices.push(calc_point(outer_radius, i));
    }

    vertices
}
