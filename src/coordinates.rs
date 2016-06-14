use std::convert::From;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::iter::Iterator;
use std::ops::Deref;

use arrayvec::ArrayVec;
use rand;
use rand::Rng;
use smallvec::{SmallVec, SmallVecMoveIterator};


pub struct DimensionSize(pub usize);

pub trait Coordinate: PartialEq + Eq + Hash + Copy + Clone + Debug + Ord + PartialOrd {

    fn from_row_major_index(index: usize, row_size: DimensionSize) -> Self;
    fn from_row_column_indices(col_index: usize, row_index: usize) -> Self;
    fn as_cartesian_2d(&self) -> Cartesian2DCoordinate;
}

pub trait Cell {

    type Coord: Coordinate;
    type Direction;
                          // Require that the Option fixed size Vec specifically wraps Coord with an Option otherwise
                          // we get type errors saying a general CoordinateOptionFixedSizeVec IntoIterator::Item cannot `unwrap`.
                          // associated type specification, not trait type parameter, but almost same syntax...
                          // e.g. FromIterator<T> is a type parameter to the trait
                          //      IntoIterator<Item=T> is an associated type specialisation
                          // Deref<Target=[Self::Coord]> gives access to the `iter` of slices.
    type CoordinateFixedSizeVec: IntoIterator<Item=Self::Coord> + FromIterator<Self::Coord> + Deref<Target=[Self::Coord]>;
    type CoordinateOptionFixedSizeVec: IntoIterator<Item=Option<Self::Coord>> + FromIterator<Option<Self::Coord>> + Deref<Target=[Option<Self::Coord>]>;
    type DirectionFixedSizeVec: IntoIterator<Item=Self::Direction> + FromIterator<Self::Direction>;

    /// Creates a small vec of the possible directions away from this Cell.
    fn offset_directions(coord: &Option<Self::Coord>) -> Self::DirectionFixedSizeVec;

    /// Creates a new `Coord` offset 1 cell away in the given direction.
    /// Returns None if the Coordinate is not representable.
    fn offset_coordinate(coord: Self::Coord, dir: Self::Direction) -> Option<Self::Coord>;

    fn rand_direction<R: Rng>(rng: &mut R) -> Self::Direction;
    fn rand_roughly_vertical_direction<R: Rng>(rng: &mut R) -> Self::Direction;
    fn rand_roughly_horizontal_direction<R: Rng>(rng: &mut R) -> Self::Direction;
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

    // do not really want this indirection as it requires me to make option itself, unwrapping a trait?
    // prefer the const 4, but they do not exist in the language yet
    // could just dynamically query the size of CoordinateFixedSizeVec? No then the option variant is not a compile time decision
    // argh
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

    fn rand_direction<R: Rng>(rng: &mut R) -> Self::Direction {
        const DIRS_COUNT: usize = 4;
        const DIRS: [CompassPrimary; DIRS_COUNT] =
            [CompassPrimary::North, CompassPrimary::South, CompassPrimary::East, CompassPrimary::West];
        let dir_index = rng.gen::<usize>() % DIRS_COUNT;
        DIRS[dir_index]
    }

    fn rand_roughly_vertical_direction<R: Rng>(rng: &mut R) -> Self::Direction {
        if rng.gen() {
            CompassPrimary::North
        } else {
            CompassPrimary::South
        }
    }
    fn rand_roughly_horizontal_direction<R: Rng>(rng: &mut R) -> Self::Direction {
        if rng.gen() {
            CompassPrimary::East
        } else {
            CompassPrimary::West
        }
    }
}


impl Cartesian2DCoordinate {
    pub fn new(x: u32, y: u32) -> Cartesian2DCoordinate {
        Cartesian2DCoordinate { x: x, y: y }
    }
}
impl Coordinate for Cartesian2DCoordinate {

    fn from_row_major_index(index: usize, row_size: DimensionSize) -> Cartesian2DCoordinate {
        let DimensionSize(size) = row_size;
        let x = index % size as usize;
        let y = index / size as usize;

        Cartesian2DCoordinate::new(x as u32, y as u32)
    }

    fn from_row_column_indices(col_index: usize, row_index: usize) -> Self {
        Cartesian2DCoordinate::new(col_index as u32, row_index as u32)
    }

    fn as_cartesian_2d(&self) -> Cartesian2DCoordinate {
        *self
    }
}

impl From<(u32, u32)> for Cartesian2DCoordinate {
    fn from(x_y_pair: (u32, u32)) -> Cartesian2DCoordinate {
        Cartesian2DCoordinate::new(x_y_pair.0, x_y_pair.1)
    }
}
