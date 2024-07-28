use std::cmp::Ordering;
use std::collections::VecDeque;

use avian2d::collision::CollidingEntities;
use avian2d::prelude::PhysicsStepSet;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::query::QueryData;
use bevy::math::NormedVectorSpace;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use itertools::Itertools;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};
use internal_shared::register_type_binder::RegisterTypeBinder;

use crate::game::platter::arm::PlatterArm;
use crate::game::platter::mesh::{PlatterMeshes, PlatterMeshOptions, PlatterSegmentMesh};
use crate::game::platter::platter::Platter;
use crate::game::platter::segment::{CenterPoint, PlatterSegment};
use crate::game::platter::spawn::{SpawnArea, SpawnAreaBundle};
use crate::game::platter::value::{BlockGrid, InnerValue, OriginType, PlatterSegmentValue};

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    // app.add_systems(Update, render.after(PhysicsStepSet::ReportContacts));
    app.add_event::<SpawnFallingBlock>();
    app.add_event::<SpawnFallingBlockFailed>();
    app.configure_sets(Update, FallingSystemSet);
    app.add_systems(Update, spawn_falling_block.in_set(FallingSystemSet));
    app.add_systems(
        Update,
        do_fall.before(spawn_falling_block).in_set(FallingSystemSet),
    );
}

#[derive(SystemSet, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FallingSystemSet;

#[derive(Event, Debug, Copy, Clone)]
pub struct SpawnFallingBlock {
    pub platter: Entity,
    pub value: InnerValue,
}

#[derive(Event, Debug, Copy, Clone)]
pub struct SpawnFallingBlockFailed {
    pub platter: Entity,
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct FallingBlock;

#[derive(RegisterTypeBinder)]
pub struct Types;

#[derive(QueryData)]
struct FallingQueryData<'w> {
    entity: Entity,
    platter_segment_value: &'w PlatterSegmentValue,
    platter_segment_mesh: &'w PlatterSegmentMesh,
}

#[derive(QueryData)]
struct SegmentQueryData<'w> {
    entity: Entity,
    platter_segment_value: &'w PlatterSegmentValue,
    platter_segment_mesh: &'w PlatterSegmentMesh,
    has_falling_block: Has<FallingBlock>,
}

fn do_fall(
    mut commands: Commands,
    falling_q: Query<FallingQueryData, (With<PlatterSegment>, With<FallingBlock>)>,
    segments_q: Query<SegmentQueryData, With<PlatterSegment>>,
) {
    let current = falling_q.iter().collect::<Vec<_>>();
    let next_unchecked = current
        .iter()
        .map(|cur| {
            let mut pie_cut = cur.platter_segment_mesh.pie_cut;
            let mut onion_layer = cur.platter_segment_mesh.onion_layer;
            onion_layer -= 1;
            let next = segments_q.iter().find(|next| {
                next.platter_segment_value.0.is_none()
                    && next.platter_segment_mesh.pie_cut == pie_cut
                    && next.platter_segment_mesh.onion_layer == onion_layer
            });
            (
                (pie_cut, onion_layer),
                cur.platter_segment_value,
                next.map(|seg| seg.entity),
            )
        })
        .collect::<Vec<_>>();
    let mut next = vec![];
    for (pos, valiue, entity_opt) in next_unchecked.into_iter() {
        let Some(next_entity) = entity_opt else {
            // TODO: cant fall anymore apply
            return;
        };
        next.push((pos, valiue, next_entity));
    }
    for item in current.into_iter() {
        commands
            .entity(item.entity)
            .remove::<FallingBlock>()
            .insert(PlatterSegmentValue::default());
    }
    for &((pie_cut, onion_layer), value, entity) in next.iter() {
        commands.entity(entity).insert((FallingBlock, *value));
    }
}

fn spawn_falling_block(
    mut commands: Commands,
    platter_q: Query<(Entity, &PlatterMeshOptions, &GlobalTransform), With<Platter>>,
    spawn_area_q: Query<&CollidingEntities, With<SpawnArea>>,
    mut segments_q: Query<
        (&PlatterSegmentMesh, &CenterPoint, Mut<PlatterSegmentValue>),
        With<PlatterSegment>,
    >,
    mut spawn_falling_block: EventReader<SpawnFallingBlock>,
    mut spawn_falling_block_failed: EventWriter<SpawnFallingBlockFailed>,
) {
    'event: for &event in spawn_falling_block.read() {
        log::debug!("SpawnFallingBlock: {event:?}");
        let Some((platter_entity, pmo, global_transform)) = platter_q.get(event.platter).ok()
        else {
            panic!("failed to find platter");
        };
        // TODO: make ti a child
        let Some(colliding) = spawn_area_q.get_single().ok() else {
            panic!("failed to find SpawnArea")
        };
        let top_row = pmo.get().onion_layers - 1;
        log::debug!("top_row: {top_row}");

        #[derive(Debug, Copy, Clone, PartialEq)]
        struct Segment {
            entity: Entity,
            layer: usize,
            slice: usize,
            center: Vec2,
            value: Option<InnerValue>,
        }

        let mut targets = HashMap::<usize, EntityHashMap<Segment>>::default();
        for &collided in colliding.0.iter() {
            let Some((psm, center, psv)) = segments_q.get(collided).ok() else {
                continue;
            };
            match event.value {
                InnerValue::RedZ
                | InnerValue::GreenS
                | InnerValue::PurpleT
                | InnerValue::BlueJ
                | InnerValue::OrangeL
                | InnerValue::YellowO => {
                    if psm.onion_layer < top_row - 3 {
                        continue;
                    }
                }
                InnerValue::CyanI => {
                    if psm.onion_layer < top_row - 4 {
                        continue;
                    }
                }
            }
            targets.entry(psm.onion_layer).or_default().insert(
                collided,
                Segment {
                    entity: collided,
                    layer: psm.onion_layer,
                    slice: psm.pie_cut,
                    center: center.get(),
                    value: psv.0,
                },
            );
        }
        let platter_x = global_transform.translation().x;
        let closest_targets = targets
            .get(&top_row)
            .expect("missing top row")
            .values()
            .sorted_by(|a, b| {
                platter_x
                    .distance_squared(a.center.x)
                    .total_cmp(&platter_x.distance_squared(b.center.x))
            })
            .collect::<Vec<_>>();

        #[derive(Debug, Copy, Clone, PartialEq)]
        enum Side {
            Left,
            Right,
        }

        let side = match closest_targets[0]
            .center
            .x
            .total_cmp(&closest_targets[1].center.x)
        {
            Ordering::Less => Side::Right,
            Ordering::Equal => unreachable!("equal distance"),
            Ordering::Greater => Side::Left,
        };

        let mut grid = match event.value.shape_coordinates().origin_type() {
            OriginType::Single(_) => {
                let mut grid_fr = [None; 3];
                let order = match side {
                    Side::Left => VecDeque::from([1, 0, 2]),
                    Side::Right => VecDeque::from([1, 2, 0]),
                };
                let mut closest = order.into_iter().zip(closest_targets.into_iter());
                while let Some((order, &value)) = closest.next() {
                    grid_fr[order] = Some(value);
                }
                let grid = [grid_fr, [None; 3], [None; 3]];
                BlockGrid::ThreeByThree(grid)
            }
            OriginType::QuadAvg(_, _, _, _) => {
                let mut grid_fr = [None; 4];
                let order = match side {
                    Side::Left => VecDeque::from([1, 2, 0, 3]),
                    Side::Right => VecDeque::from([2, 1, 3, 0]),
                };
                let mut closest = order.into_iter().zip(closest_targets.into_iter());
                while let Some((order, &value)) = closest.next() {
                    grid_fr[order] = Some(value);
                }
                let grid = [grid_fr, [None; 4], [None; 4], [None; 4]];
                BlockGrid::FourByFour(grid)
            }
        };

        fn populate<const W: usize, const H: usize>(
            grid: &mut [[Option<Segment>; W]; H],
            targets: &HashMap<usize, EntityHashMap<Segment>>,
        ) {
            let Some(&top_row) = targets.keys().max() else {
                panic!("empty targets");
            };
            let fr = grid[0];
            for (row_ix, columns) in (0..grid.len() - 1).map(|c| (c + 1, fr)) {
                for (col_ix, column) in columns.iter().enumerate() {
                    let Some(column) = column else {
                        unreachable!();
                    };
                    let ix = top_row - row_ix;
                    let value = targets
                        .get(&ix)
                        .expect("missing row")
                        .values()
                        .find(|&seg| top_row - seg.layer == row_ix && seg.slice == column.slice)
                        .copied();
                    let Some(value) = value else {
                        panic!("empty value for layer {ix}")
                    };
                    log::debug!("setting [{row_ix}][{col_ix}] = {value:?}");
                    debug_assert!(grid[row_ix][col_ix].is_none());
                    grid.get_mut(row_ix)
                        .unwrap_or_else(|| unreachable!())
                        .get_mut(col_ix)
                        .unwrap_or_else(|| unreachable!())
                        .replace(value);
                    assert_eq!(grid[row_ix][col_ix], Some(value));
                }
            }
        }

        match grid {
            BlockGrid::ThreeByThree(ref mut grid) => {
                populate(grid, &targets);
            }
            BlockGrid::ThreeByFour(ref mut grid) => {
                populate(grid, &targets);
            }
            BlockGrid::FourByFour(ref mut grid) => {
                populate(grid, &targets);
            }
        }

        let shape_grid = event.value.shape_coordinates();
        debug_assert!(grid.is_same_size(&shape_grid));

        for (row_ix, row) in grid.iter().enumerate() {
            for (col_ix, col) in row.iter().enumerate() {
                let Some(seg) = col else {
                    panic!("bad state: {row_ix}, {col_ix}");
                };
                if shape_grid.get(row_ix, col_ix) {
                    if seg.value.is_some() {
                        spawn_falling_block_failed.send(SpawnFallingBlockFailed {
                            platter: platter_entity,
                        });
                    }
                }
            }
        }

        for (row_ix, row) in grid.iter().enumerate() {
            for (col_ix, col) in row.iter().enumerate() {
                let Some(seg) = col else {
                    panic!("bad state: {row_ix}, {col_ix}");
                };
                if shape_grid.get(row_ix, col_ix) {
                    commands
                        .entity(seg.entity)
                        .insert(FallingBlock)
                        .insert(PlatterSegmentValue(Some(event.value)));
                }
            }
        }
    }
}

fn render(
    mut commands: Commands,
    arm_q: Query<(Entity, &CollidingEntities), With<PlatterArm>>,
    mut segments_q: Query<&mut PlatterSegmentValue, With<PlatterSegment>>,
) {
    for (entity, colliding_entities) in arm_q.iter() {
        for &colliding_entity in colliding_entities.0.iter() {
            let Some(mut psv) = segments_q.get_mut(colliding_entity).ok() else {
                continue;
            };
            psv.0.replace(InnerValue::RedZ);
        }
    }
}
