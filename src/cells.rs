use std::convert::From;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::iter::Iterator;
use std::ops::Deref;

use rand::{Rng, XorShiftRng};
use smallvec::SmallVec;

use gridTraits::{GridIterators, GridDisplay, GridDimensions, GridPositions};
use units::{RowLength, RowIndex, ColumnLength, ColumnIndex};

pub trait Coordinate: PartialEq + Eq + Hash + Copy + Clone + Debug + Ord + PartialOrd {

    fn from_row_major_index(index: usize, data: &GridDimensions) -> Self;
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

    fn rand_direction(rng: &mut XorShiftRng) -> Self::Direction;
    fn rand_roughly_vertical_direction(rng: &mut XorShiftRng) -> Self::Direction;
    fn rand_roughly_horizontal_direction(rng: &mut XorShiftRng) -> Self::Direction;
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

    fn rand_direction(rng: &mut XorShiftRng) -> Self::Direction {
        const DIRS_COUNT: usize = 4;
        const DIRS: [CompassPrimary; DIRS_COUNT] =
            [CompassPrimary::North, CompassPrimary::South, CompassPrimary::East, CompassPrimary::West];
        let dir_index = rng.gen::<usize>() % DIRS_COUNT;
        DIRS[dir_index]
    }

    fn rand_roughly_vertical_direction(rng: &mut XorShiftRng) -> Self::Direction {
        if rng.gen() {
            CompassPrimary::North
        } else {
            CompassPrimary::South
        }
    }
    fn rand_roughly_horizontal_direction(rng: &mut XorShiftRng) -> Self::Direction {
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

    #[inline]
    fn from_row_major_index(index: usize, data: &GridDimensions) -> Cartesian2DCoordinate {
        let RowLength(width) = data.row_length(None).expect("invalid row index"); // todo fix up for Polar mazes
        let x = index % width;
        let y = index / width;

        Cartesian2DCoordinate::new(x as u32, y as u32)
    }

    #[inline]
    fn from_row_column_indices(col_index: ColumnIndex, row_index: RowIndex) -> Self {
        let (ColumnIndex(col), RowIndex(row)) = (col_index, row_index);
        Cartesian2DCoordinate::new(col as u32, row as u32)
    }

    #[inline(always)]
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
    Outward //(u8) // 0, 1
}

impl Cell for PolarCell {

    type Coord = Cartesian2DCoordinate;
    type Direction = ClockDirection;
    type CoordinateSmallVec = SmallVec<[Self::Coord; 8]>;
    type CoordinateOptionSmallVec = SmallVec<[Option<Self::Coord>; 8]>;
    type DirectionSmallVec = SmallVec<[Self::Direction; 8]>;

    /// Creates a small vec of the possible directions away from this Cell.
    fn offset_directions(_: &Option<Self::Coord>) -> Self::DirectionSmallVec {
        [ClockDirection::Clockwise,
         ClockDirection::CounterClockwise,
         ClockDirection::Inward,
         ClockDirection::Outward]
        .into_iter().cloned().collect::<Self::DirectionSmallVec>()

        // and extend from the outward direction instance? [ClockDirection::Outward, ClockDirection::Outward, ClockDirection::Outward]?
    }

    /// Creates a new `Coord` offset 1 cell away in the given direction.
    /// Returns None if the Coordinate is not representable.
    fn offset_coordinate(_: Self::Coord, dir: Self::Direction) -> Option<Self::Coord> {

        //let (x, y) = (coord.x, coord.y);
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

    fn rand_direction(rng: &mut XorShiftRng) -> Self::Direction { // what about multiple outward options? outward is not a single direction
        const DIRS_COUNT: usize = 4;
        const DIRS: [ClockDirection; DIRS_COUNT] =
            [ClockDirection::Clockwise, ClockDirection::CounterClockwise, ClockDirection::Inward, ClockDirection::Outward];
        let dir_index = rng.gen::<usize>() % DIRS_COUNT;
        DIRS[dir_index]
    }

    fn rand_roughly_vertical_direction(rng: &mut XorShiftRng) -> Self::Direction {
        if rng.gen() {
            ClockDirection::Clockwise
        } else {
            ClockDirection::CounterClockwise
        }
    }

    fn rand_roughly_horizontal_direction(rng: &mut XorShiftRng) -> Self::Direction {
        if rng.gen() {
            ClockDirection::Inward
        } else {
            ClockDirection::Outward
        }
    }

}

// Polar grid constructor
// For any coord[x][y]
// what are the neighbours? - what coordinates and handle outward[n]
// what is the corresponding graph index?
// coord.x
// coord.y            // grid config data access - number of rings
// row_height = 1.0 / grid.rows_count     # e.g. row 1. 0.25
// radius = y * (row_height as float)     # 0.25
// circumference = 2 * pi * radius        # 1.5707

// previous_rows_count = .. # 1
// cell_width_if_using_same_cell_count_as_previous_ring = circumference / previous_rows_count  # 1.5707
// ratio = round to nearest number (cell_width_if_using_same_cell_count_as_previous_ring / row_height)  # 6.28 -> 6.0

// cells = previous_rows_count * ratio # 6

// // now we know the cell count in the ring
// // varying cell counts per row
// // coord[x][y]
// // statically the row length is unknown, but the number of rings is fixed when creating the grid.
// // ok, so now we know the row length...
// for any cell on the row, if not row 0 (only 1 cell)
// cw = coord[x+1][y]
// ccw = coord[x-1][y] (wrapping around?)
// // what is previous row length ?
// ratio = row_length / previous_row_length
// parent = cell[x/ratio][y-1]
// parent.outward += this // so to calculate outward of *this*, argh...
// inward = parent // easy to calc inward

// Data needed:
// - number of rows (y height) of the grid
// - row length of previous row [y-1], which varies on each ring - so length of all rows
//   same as current row or half the length
// - outward cells??? max 2 parents per cell I think. If 1 parent then cell count is the same as current row.
//                    if 2 parents then 2 * current rows cells count and outward = [x * 2][y+1] and [(x*2)+1][y+1]
//
// *so row lengths and number of rows is most pertinent*

// - `offset_directions` needs this data to know how many outward cells there are
// - `offset_coordinate` needs to calculate inward, which could be done by knowing how many inward cells there are
//                       Clockwise/CounterClockwise need to know row length to wrap or not allow wrapping.
// where to store grid rows count and length of each row?

// How to map coordinate to graph index? Need new coordinate type to have a different mapping function.
// coord[x][y] == ? Also needs to look up the length of each row. Might be nice to prefix sum the row lengths at each point.
// The lengths need to be calced once at, presumably at a similar time to when we decide the number of nodes in the graph.
//
// refactoring `Grid` (?) for Polar
// rows: RowsCount,
// columns: ColumnsCount,
// still need these but the user should not be providing columns as an argument, or it should be ignored
// - additional data required for rowLengths
// - cells count needs deciding before creating the graph and adding nodes.
// - `size` = (rows * columns) must be customised
// - `random_cell` assumes a fixed row*col size, but the dimensions vary.
// - `grid_coordinate_to_index` assumes a fixed row*col size
// - `CellIter` assumes a fixed row*col size

// must aggregate, how?
// static dispatch (generic parameters) vs dynamic &TraitX
// dynamic: injection of trait objects by reference/box/rc?
//    dynamic trait indirection via pointer probably not too bad overhead,
//    at least in terms of data locality as most data is in the graph. Other heap allocated data
//    would be close together and small.

// trait GridData<CellT: Cell>  {
//     /// The length of a particular row
//     fn rowLength(RowIndex rowIndex) -> Option<RowLength>;
//     fn size() -> usize;
//     fn rows(&self) -> RowsCount;
//     fn row_length(&self) -> RowLength;
//     fn columns(&self) -> ColumnsCount;
//     fn column_length(&self) -> ColumnLength;

//     fn grid_coordinate_to_index(coord: CellT::Coord) -> Option<usize>; /// ???
//     fn random_cell(&self, rng: &mut XorShiftRng) -> CellT::Coord;

//     fn graphSize(&self) -> (usize, usize); // (node hint, edges hint)

//     //     let cells_count = self.size();
//     //     let nodes_hint = _;
//     //     let edges_hint = _;
//     // }

//     // We need a GridData instance before we can create the Grid instance
//     //
// }

// struct Grid {

//     // Cell trait needs to take an Option<&GridData>
//     // offset_coordinate
//     // offset_directions

//     fn new(gd: GridData) {
//         ...
//     }
// }

// trait GridIterators<CellT: Cell, GridDimensions: GridData> {

//     type CellIter: Iterator<Item=CellT::Coord>;
//     type BatchIter: Iterator<Item=Vec<CellT::Coord>>;
//     fn iter(&GridDimensions) -> CellIter;
//     fn iter_row(&GridDimensions) -> BatchIter;
//     fn iter_column(&GridDimensions) -> BatchIter;
// }


// - traits need not contain data, but may have to be PhantomData if that is the case.
// Grid
//     GridIterators
//     GridData

// https://www.youtube.com/watch?v=jGNNazG8yyk

