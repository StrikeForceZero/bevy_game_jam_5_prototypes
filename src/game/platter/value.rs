use bevy::color::palettes::css::*;
use bevy::color::palettes::tailwind::{CYAN_100, CYAN_200, CYAN_300};
use bevy::prelude::*;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::platter::mesh::PlatterSegmentMesh;
use crate::game::platter::segment::PlatterSegmentColor;
use crate::util::color_material_manager::{AssociatedColorMaterial, ColorMaterialManagerId};
use crate::util::ref_ext::RefExt;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(Update, platter_value_updated);
}

#[derive(Debug)]
pub enum BlockGrid<T> {
    ThreeByThree([[T; 3]; 3]),
    ThreeByFour([[T; 4]; 3]),
    FourByFour([[T; 4]; 4]),
}

impl<T> BlockGrid<T>
where
    T: Default + Copy + Clone,
{
    pub fn rotate(&self) -> Self {
        match self {
            Self::ThreeByThree(grid) => {
                let mut rotated = [[T::default(); 3]; 3];

                for i in 0..3 {
                    for j in 0..3 {
                        rotated[j][2 - i] = grid[i][j];
                    }
                }

                Self::ThreeByThree(rotated)
            }
            Self::ThreeByFour(grid) => {
                // no rotation
                Self::ThreeByFour(grid.clone())
            }
            Self::FourByFour(grid) => {
                let mut rotated = [[T::default(); 4]; 4];

                for i in 0..4 {
                    for j in 0..4 {
                        rotated[j][3 - i] = grid[i][j];
                    }
                }

                Self::FourByFour(rotated)
            }
        }
    }
    pub fn get(&self, row: usize, column: usize) -> T {
        match self {
            BlockGrid::ThreeByThree(grid) => grid[row][column],
            BlockGrid::ThreeByFour(grid) => grid[row][column],
            BlockGrid::FourByFour(grid) => grid[row][column],
        }
    }
    pub fn iter(&self) -> BlockGridIterator<T> {
        BlockGridIterator::new(self)
    }
    pub fn is_same_size<T2>(&self, rhs: &BlockGrid<T2>) -> bool {
        match (self, rhs) {
            (Self::ThreeByThree(_), BlockGrid::ThreeByThree(_))
            | (Self::ThreeByFour(_), BlockGrid::ThreeByFour(_))
            | (Self::FourByFour(_), BlockGrid::FourByFour(_)) => true,
            (Self::ThreeByThree(_), _) | (Self::ThreeByFour(_), _) | (Self::FourByFour(_), _) => {
                false
            }
            _ => panic!("non exhaustive"),
        }
    }
}

pub enum OriginType {
    Single(UVec2),
    QuadAvg(UVec2, UVec2, UVec2, UVec2),
}

pub enum Origin {
    Single(UVec2),
    QuadAvg(URect),
}

impl<T> BlockGrid<T> {
    pub fn origin_type(&self) -> OriginType {
        match self {
            Self::ThreeByThree(_) => {
                // [ ][ ][ ]
                // [ ][x][ ]
                // [ ][ ][ ]
                OriginType::Single(UVec2::new(1, 1))
            }
            Self::ThreeByFour(_) => {
                // [ ][x][x][ ]
                // [ ][x][x][ ]
                // [ ][ ][ ][ ]
                OriginType::QuadAvg(
                    UVec2::new(1, 0),
                    UVec2::new(1, 1),
                    UVec2::new(2, 0),
                    UVec2::new(2, 1),
                )
            }
            Self::FourByFour(_) => {
                // [ ][ ][ ][ ]
                // [ ][x][x][ ]
                // [ ][x][x][ ]
                // [ ][ ][ ][ ]
                OriginType::QuadAvg(
                    UVec2::new(1, 1),
                    UVec2::new(1, 2),
                    UVec2::new(2, 1),
                    UVec2::new(2, 2),
                )
            }
        }
    }
}

impl BlockGrid<UVec2> {
    pub fn origin(&self) -> Origin {
        match self {
            Self::ThreeByThree(grid) => {
                // [ ][ ][ ]
                // [ ][x][ ]
                // [ ][ ][ ]
                Origin::Single(grid[1][1])
            }
            Self::ThreeByFour(grid) => {
                // [ ][x][x][ ]
                // [ ][x][x][ ]
                // [ ][ ][ ][ ]
                Origin::QuadAvg(URect::from_corners(grid[0][1], grid[1][2]))
            }
            Self::FourByFour(grid) => {
                // [ ][ ][ ][ ]
                // [ ][x][x][ ]
                // [ ][x][x][ ]
                // [ ][ ][ ][ ]
                Origin::QuadAvg(URect::from_corners(grid[1][1], grid[2][2]))
            }
        }
    }
}

pub struct BlockGridIterator<'a, T> {
    grid: &'a BlockGrid<T>,
    row: usize,
}

impl<'a, T> BlockGridIterator<'a, T> {
    pub fn new(grid: &'a BlockGrid<T>) -> Self {
        BlockGridIterator::<T> { grid, row: 0 }
    }
}

impl<'a, T> Iterator for BlockGridIterator<'a, T>
where
    T: Clone,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.grid {
            BlockGrid::ThreeByThree(grid) => {
                if self.row > 2 {
                    None
                } else {
                    Some(grid[self.row].to_vec())
                }
            }
            BlockGrid::ThreeByFour(grid) => {
                if self.row > 3 {
                    None
                } else {
                    Some(grid[self.row].to_vec())
                }
            }
            BlockGrid::FourByFour(grid) => {
                if self.row > 3 {
                    None
                } else {
                    Some(grid[self.row].to_vec())
                }
            }
        };
        if res.is_some() {
            self.row += 1;
        }
        res
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Reflect, AutoRegisterType)]
pub enum InnerValue {
    RedZ,
    GreenS,
    YellowO,
    PurpleT,
    BlueJ,
    OrangeL,
    CyanI,
}

impl InnerValue {
    pub fn color(&self) -> Color {
        match self {
            InnerValue::RedZ => RED.into(),
            InnerValue::GreenS => GREEN.into(),
            InnerValue::YellowO => YELLOW.into(),
            InnerValue::PurpleT => PURPLE.into(),
            InnerValue::BlueJ => BLUE.into(),
            InnerValue::OrangeL => ORANGE.into(),
            InnerValue::CyanI => CYAN_300.into(),
        }
    }
    pub fn shape_coordinates(&self) -> BlockGrid<bool> {
        const T: bool = true;
        const F: bool = false;
        match self {
            InnerValue::RedZ => BlockGrid::ThreeByThree([[T, T, F], [F, T, T], [F, F, F]]),
            InnerValue::GreenS => BlockGrid::ThreeByThree([[F, T, T], [T, T, F], [F, F, F]]),
            InnerValue::YellowO => {
                BlockGrid::ThreeByFour([[F, T, T, F], [F, T, T, F], [F, F, F, F]])
            }
            InnerValue::PurpleT => BlockGrid::ThreeByThree([[F, T, F], [T, T, T], [F, F, F]]),
            InnerValue::BlueJ => BlockGrid::ThreeByThree([[T, F, F], [T, T, T], [F, F, F]]),
            InnerValue::OrangeL => BlockGrid::ThreeByThree([[F, F, T], [T, T, T], [F, F, F]]),
            InnerValue::CyanI => {
                BlockGrid::FourByFour([[F, F, F, F], [T, T, T, T], [F, F, F, F], [F, F, F, F]])
            }
        }
    }
}

impl AssociatedColorMaterial for InnerValue {
    fn get_id(&self) -> ColorMaterialManagerId {
        self.color().get_id()
    }

    fn get_color_material(&self) -> ColorMaterial {
        self.color().get_color_material()
    }
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterSegmentValue(pub Option<InnerValue>);

#[derive(RegisterTypeBinder)]
pub struct Types;

fn platter_value_updated(
    mut changed: Query<
        (
            Entity,
            Ref<PlatterSegmentValue>,
            &PlatterSegmentMesh,
            Mut<PlatterSegmentColor>,
        ),
        (Changed<PlatterSegmentValue>, With<PlatterSegmentColor>),
    >,
) {
    for (entity, value, psm, mut psc) in changed.iter_mut() {
        if !value.is_added_or_changed() {
            continue;
        }
        let new_color = match value.0 {
            None => psm.options.initial_segment_color,
            Some(inner) => inner.color(),
        };
        psc.0 = new_color;
    }
}
