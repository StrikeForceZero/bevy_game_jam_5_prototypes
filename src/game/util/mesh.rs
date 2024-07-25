use std::f32::consts::PI;

use bevy::math::Vec3;
use bevy::prelude::{Mesh, Vec2};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

fn ear_clip_triangulate(vertices: &[Vec2]) -> Vec<[usize; 3]> {
    let mut indices: Vec<[usize; 3]> = Vec::new();
    let mut remaining: Vec<usize> = (0..vertices.len()).collect();

    while remaining.len() > 3 {
        let mut i = 0;
        while i < remaining.len() {
            let prev = remaining[(i + remaining.len() - 1) % remaining.len()];
            let curr = remaining[i];
            let next = remaining[(i + 1) % remaining.len()];

            if is_ear(vertices, &remaining, prev, curr, next) {
                indices.push([prev, curr, next]);
                remaining.remove(i);
                break;
            }

            i += 1;
        }
    }

    indices.push([remaining[0], remaining[1], remaining[2]]);

    indices
}

fn is_ear(vertices: &[Vec2], remaining: &[usize], prev: usize, curr: usize, next: usize) -> bool {
    if !is_convex(vertices, prev, curr, next) {
        return false;
    }

    for &idx in remaining {
        if idx != prev
            && idx != curr
            && idx != next
            && point_in_triangle(
                vertices[idx],
                vertices[prev],
                vertices[curr],
                vertices[next],
            )
        {
            return false;
        }
    }

    true
}

fn is_convex(vertices: &[Vec2], prev: usize, curr: usize, next: usize) -> bool {
    let p = vertices[prev];
    let c = vertices[curr];
    let n = vertices[next];

    (c.y - p.y) * (n.x - c.x) - (n.y - c.y) * (c.x - p.x) >= 0.0
}

fn point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let p = point;
    let area = 0.5 * (-b.y * c.x + a.y * (-b.x + c.x) + a.x * (b.y - c.y) + b.x * c.y);
    let s = 1.0 / (2.0 * area) * (a.y * c.x - a.x * c.y + (c.y - a.y) * p.x + (a.x - c.x) * p.y);
    let t = 1.0 / (2.0 * area) * (a.x * b.y - a.y * b.x + (a.y - b.y) * p.x + (b.x - a.x) * p.y);

    s > 0.0 && t > 0.0 && (s + t) < 1.0
}

pub fn calculate_centroid(vertices: &[Vec2]) -> Vec2 {
    let sum = vertices
        .iter()
        .fold(Vec2::ZERO, |acc, &vertex| acc + vertex);
    sum / vertices.len() as f32
}

pub fn close_line_strip(vertices: &mut Vec<Vec2>) {
    let &first = vertices.first().expect("line strip empty");
    if Some(&first) == vertices.last() {
        return;
    }
    vertices.push(first)
}

pub fn line_strip_2d_to_mesh(mut vertices: Vec<Vec2>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    close_line_strip(&mut vertices);
    let indices = ear_clip_triangulate(&vertices).into_flattened();
    let vertices = vertices
        .into_iter()
        .map(|v| v.extend(0.0))
        .collect::<Vec<_>>();
    let normals = (0..vertices.len()).map(|_| Vec3::Z).collect::<Vec<_>>();
    let uvs = (0..vertices.len()).map(|_| Vec2::ONE).collect::<Vec<_>>();
    mesh.insert_indices(Indices::U32(
        // TODO: this is messy
        indices.into_iter().map(|v| v as u32).collect(),
    ));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

pub fn line_strip_into_triangle_list_indices(vertices: &[Vec3]) -> Vec<u32> {
    let mut indices = Vec::new();
    for i in 1..vertices.len() {
        indices.push(i as u32 - 1);
        indices.push(i as u32);
        indices.push(i as u32);

        indices.push(i as u32 - 1);
        indices.push(i as u32);
        indices.push(i as u32 - 1);
    }
    indices
}

pub fn generate_donut_vertices(
    inner_radius: f32,
    outer_radius: f32,
    inner_resolution: usize,
    outer_resolution: usize,
) -> Vec<Vec2> {
    let mut vertices = Vec::new();

    let calc_point = |radius: f32, index: usize, resolution: usize| -> Vec2 {
        if index > resolution {
            panic!("invalid index got: {index}, expected: 0..={resolution}");
        }
        let theta = 2.0 * PI * index as f32 / resolution as f32;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        Vec2::new(x, y)
    };

    // Generate inside circle vertices
    for i in 0..=inner_resolution {
        vertices.push(calc_point(inner_radius, i, inner_resolution));
    }

    // Generate outside circle vertices
    // Going in reverse will ensure the inner start and stop match the outers
    for i in (0..=outer_resolution).rev() {
        vertices.push(calc_point(outer_radius, i, outer_resolution));
    }

    vertices
}

pub fn generate_subdivided_donut_split_vertices(
    inner_radius: f32,
    outer_radius: f32,
    inner_resolution: usize,
    outer_resolution: usize,
    pie_cuts: usize,
    onion_rings: usize,
    close_segments: bool,
) -> Vec<Vec<Vec<Vec2>>> {
    let mut vertices: Vec<Vec<Vec<Vec2>>> = Vec::new();

    let angle_step = 2.0 * PI / pie_cuts as f32;
    let ring_step = (outer_radius - inner_radius) / onion_rings as f32;

    for i in 0..pie_cuts {
        let angle_start = i as f32 * angle_step;
        let angle_end = (i + 1) as f32 * angle_step;

        let mut pie_segment: Vec<Vec<Vec2>> = Vec::new();

        for j in 0..onion_rings {
            let radius_start = inner_radius + j as f32 * ring_step;
            let radius_end = inner_radius + (j + 1) as f32 * ring_step;

            let mut layer_vertices: Vec<Vec2> = Vec::new();

            // Generate vertices for the start arc (radius_start)
            for k in 0..=inner_resolution {
                let theta =
                    angle_start + k as f32 * (angle_end - angle_start) / inner_resolution as f32;
                let x = radius_start * theta.cos();
                let y = radius_start * theta.sin();
                layer_vertices.push(Vec2::new(x, y));
            }

            // Generate vertices for the end arc (radius_end)
            for l in (0..=outer_resolution).rev() {
                let theta =
                    angle_start + l as f32 * (angle_end - angle_start) / outer_resolution as f32;
                let x = radius_end * theta.cos();
                let y = radius_end * theta.sin();
                layer_vertices.push(Vec2::new(x, y));
            }

            if close_segments {
                close_line_strip(&mut layer_vertices);
            }

            pie_segment.push(layer_vertices);
        }

        vertices.push(pie_segment);
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_centroid() {
        let params = [
            (vec![Vec2::X], Vec2::X),
            (vec![Vec2::X, Vec2::Y], Vec2::new(0.5, 0.5)),
            (vec![Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y], Vec2::ZERO),
        ];
        for (input, expected) in params {
            assert_eq!(
                calculate_centroid(&input),
                expected,
                "wrong centroid for: {input:?}"
            );
        }
    }
}
