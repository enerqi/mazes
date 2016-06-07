use std::convert::From;
use std::iter::FromIterator;
use std::iter::Iterator;

use arrayvec::ArrayVec;
use smallvec::{SmallVec, SmallVecMoveIterator};


pub trait Cell {

    type Coord: Coordinate;
    type Direction;
    type CoordinateFixedSizeVec: IntoIterator + FromIterator<Self::Coord>;
    type CoordinateOptionFixedSizeVec: IntoIterator + FromIterator<Option<Self::Coord>>;
    type DirectionFixedSizeVec: IntoIterator + FromIterator<Self::Direction>;

    /// Creates a small vec of the possible directions away from this Cell.
    fn offset_directions(coord: &Option<Self::Coord>) -> Self::DirectionFixedSizeVec;

    /// Creates a new `Coord` offset 1 cell away in the given direction.
    /// Returns None if the Coordinate is not representable.
    fn offset_coordinate(coord: Self::Coord, dir: Self::Direction) -> Option<Self::Coord>;
}

pub trait Coordinate {

    fn as_cartesian_2d(&self) -> Cartesian2DCoordinate;
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug, Ord, PartialOrd)]
pub struct Cartesian2DCoordinate {
    pub x: u32,
    pub y: u32,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CompassPrimary {
    North,
    South,
    East,
    West,
}

#[derive(Copy, Clone, Debug)]
pub struct SquareCell;

impl Cell for SquareCell {
    type Coord = Cartesian2DCoordinate;  // : Debug, Copy, Clone
    type Direction = CompassPrimary;
    type CoordinateFixedSizeVec = ArrayVec<[Self::Coord; 4]>;
    type CoordinateOptionFixedSizeVec = ArrayVec<[Option<Self::Coord>; 4]>;
    type DirectionFixedSizeVec = ArrayVec<[CompassPrimary; 4]>;

    fn offset_directions(coord: &Option<Self::Coord>) -> Self::DirectionFixedSizeVec {
        [CompassPrimary::North,
         CompassPrimary::South,
         CompassPrimary::East,
         CompassPrimary::West]
        .into_iter().cloned().collect::<Self::DirectionFixedSizeVec>()
    }

    fn offset_coordinate(coord: Self::Coord, dir: Self::Direction) -> Option<Self::Coord> {

        let (x, y) = (coord.x, coord.y);
        match dir {
            CompassPrimary::North => {
                if y > 0 {
                    Some(Cartesian2DCoordinate { x: x, y: y - 1 })
                } else {
                    None
                }
            }
            CompassPrimary::South => Some(Cartesian2DCoordinate { x: x, y: y + 1 }),
            CompassPrimary::East => Some(Cartesian2DCoordinate { x: x + 1, y: y }),
            CompassPrimary::West => {
                if x > 0 {
                    Some(Cartesian2DCoordinate { x: x - 1, y: y })
                } else {
                    None
                }
            }
        }
    }
}


impl Cartesian2DCoordinate {
    pub fn new(x: u32, y: u32) -> Cartesian2DCoordinate {
        Cartesian2DCoordinate { x: x, y: y }
    }
}
impl Coordinate for Cartesian2DCoordinate {

    fn as_cartesian_2d(&self) -> Cartesian2DCoordinate {
        *self
    }
}

impl From<(u32, u32)> for Cartesian2DCoordinate {
    fn from(x_y_pair: (u32, u32)) -> Cartesian2DCoordinate {
        Cartesian2DCoordinate::new(x_y_pair.0, x_y_pair.1)
    }
}
