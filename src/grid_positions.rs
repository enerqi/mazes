use std::marker::PhantomData;
use std::fmt;
use std::rc::Rc;

use rand::XorShiftRng;

use cells::{Cell, Coordinate};
use grid_traits::{GridDimensions, GridIterators};
use units::{RowsCount, RowLength, RowIndex, ColumnsCount, ColumnLength,
            ColumnIndex};


#[derive(Debug, Copy, Clone)]
pub struct RectGridPositions;

impl<CellT: Cell> for RectGridPositions {

    fn grid_coordinate_to_index(&self, coord: CellT::Coord, dimensions: &GridDimensions) -> Option<usize> {

        let grid_2d_coord = coord.as_cartesian_2d();
        if self.is_valid_coordinate(grid_2d_coord) { // also used in the general grid code = row/col_length = Dimensions
            let RowLength(row_size) = self.row_length();
            Some((grid_2d_coord.y as usize * row_size) + grid_2d_coord.x as usize)
        } else {
            None
        }
    }

    fn random_cell(&self, rng: &mut XorShiftRng,  dimensions: &GridDimensions) -> CellT::Coord {

        let index = rng.gen::<usize>() % dimensions.size();
        CellT::Coord::from_row_major_index(index, dimensions.row_length(), dimensions.column_length())
    }
}
