use std::convert::From;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::iter::Iterator;
use std::ops::Deref;

use rand::Rng;
use smallvec::SmallVec;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RowsCount(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColumnsCount(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RowLength(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColumnLength(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RowIndex(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColumnIndex(pub usize);

pub trait Coordinate: PartialEq + Eq + Hash + Copy + Clone + Debug + Ord + PartialOrd {

    fn from_row_major_index(index: usize, row_size: RowLength, column_size: ColumnLength) -> Self;
    fn from_row_column_indices(col_index: ColumnIndex, row_index: RowIndex) -> Self;
    fn as_cartesian_2d(&self) -> Cartesian2DCoordinate;
}

pub trait Cell {

    type Coord: Coordinate;
    type Direction: Eq + PartialEq + Copy + Clone + Debug;
                          // Require that the Option fixed size Vec specifically wraps Coord with an Option otherwise
                          // we get type errors saying a general CoordinateOptionSmallVec IntoIterator::Item cannot `unwrap`.
                          // associated type specification, not trait type parameter, but almost same syntax...
                          // e.g. FromIterator<T> is a type parameter to the trait
                          //      IntoIterator<Item=T> is an associated type specialisation
                          // Deref<Target=[Self::Coord]> gives access to the `iter` of slices.
    type CoordinateSmallVec: FromIterator<Self::Coord> + Deref<Target=[Self::Coord]>;
    type CoordinateOptionSmallVec: FromIterator<Option<Self::Coord>> + Deref<Target=[Option<Self::Coord>]>;
    type DirectionSmallVec: FromIterator<Self::Direction> + Deref<Target=[Self::Direction]>;

    /// Creates a small vec of the possible directions away from this Cell.
    fn offset_directions(coord: &Option<Self::Coord>) -> Self::DirectionSmallVec;

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
    type CoordinateSmallVec = SmallVec<[Self::Coord; 4]>;

    // do not really want this indirection as it requires me to make option itself, unwrapping a trait?
    // prefer the const 4, but they do not exist in the language yet
    // could just dynamically query the size of CoordinateSmallVec? No then the option variant is not a compile time decision
    // argh
    type CoordinateOptionSmallVec = SmallVec<[Option<Self::Coord>; 4]>;

    type DirectionSmallVec = SmallVec<[CompassPrimary; 4]>;

    fn offset_directions(_: &Option<Self::Coord>) -> Self::DirectionSmallVec {
        [CompassPrimary::North,
         CompassPrimary::South,
         CompassPrimary::East,
         CompassPrimary::West]
        .into_iter().cloned().collect::<Self::DirectionSmallVec>()
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

    fn from_row_major_index(index: usize, row_size: RowLength, _: ColumnLength) -> Cartesian2DCoordinate {
        let RowLength(width) = row_size;
        let x = index % width;
        let y = index / width;

        Cartesian2DCoordinate::new(x as u32, y as u32)
    }

    fn from_row_column_indices(col_index: ColumnIndex, row_index: RowIndex) -> Self {
        let (ColumnIndex(col), RowIndex(row)) = (col_index, row_index);
        Cartesian2DCoordinate::new(col as u32, row as u32)
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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PolarCell;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ClockDirection {
    Clockwise,
    CounterClockwise,
    Inward,
    Outward
}

impl Cell for PolarCell {

    type Coord = Cartesian2DCoordinate;
    type Direction = ClockDirection;
    type CoordinateSmallVec = SmallVec<[Self::Coord; 8]>;
    type CoordinateOptionSmallVec = SmallVec<[Option<Self::Coord>; 8]>;
    type DirectionSmallVec = SmallVec<[Self::Direction; 8]>;

    /// Creates a small vec of the possible directions away from this Cell.
    fn offset_directions(coord: &Option<Self::Coord>) -> Self::DirectionSmallVec {
        [ClockDirection::Clockwise,
         ClockDirection::CounterClockwise,
         ClockDirection::Inward,
         ClockDirection::Outward]
        .into_iter().cloned().collect::<Self::DirectionSmallVec>()

        // and extend from the outward direction instance? [ClockDirection::Outward, ClockDirection::Outward, ClockDirection::Outward]?
    }

    /// Creates a new `Coord` offset 1 cell away in the given direction.
    /// Returns None if the Coordinate is not representable.
    fn offset_coordinate(coord: Self::Coord, dir: Self::Direction) -> Option<Self::Coord> {

        let (x, y) = (coord.x, coord.y);
        match dir {
            ClockDirection::Clockwise => {

            }
            ClockDirection::CounterClockwise => {},
            ClockDirection::Inward => {},
            ClockDirection::Outward => {

            }
        };

        None
    }

    fn rand_direction<R: Rng>(rng: &mut R) -> Self::Direction { // what about multiple outward options? outward is not a single direction
        const DIRS_COUNT: usize = 4;
        const DIRS: [ClockDirection; DIRS_COUNT] =
            [ClockDirection::Clockwise, ClockDirection::CounterClockwise, ClockDirection::Inward, ClockDirection::Outward];
        let dir_index = rng.gen::<usize>() % DIRS_COUNT;
        DIRS[dir_index]
    }

    fn rand_roughly_vertical_direction<R: Rng>(rng: &mut R) -> Self::Direction {
        if rng.gen() {
            ClockDirection::Clockwise
        } else {
            ClockDirection::CounterClockwise
        }
    }

    fn rand_roughly_horizontal_direction<R: Rng>(rng: &mut R) -> Self::Direction {
        if rng.gen() {
            ClockDirection::Inward
        } else {
            ClockDirection::Outward
        }
    }

}

// Polar grid constructor
// number of rows, assume "columns" is 1, which will be adaptively subdivided.
// so need a RectGrid where can specify the rows and columns

