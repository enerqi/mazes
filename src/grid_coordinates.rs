use std::rc::Rc;

use rand::{Rng, XorShiftRng};

use cells::{Cartesian2DCoordinate, Cell, Coordinate};
use grid_traits::{GridDimensions, GridCoordinates};
use units::RowLength;


#[derive(Debug, Copy, Clone)]
pub struct RectGridPositions;

impl<CellT: Cell> GridCoordinates<CellT> for RectGridPositions {

    fn grid_coordinate_to_index(&self, coord: CellT::Coord, dimensions: &Rc<GridDimensions>) -> Option<usize> {

        if GridCoordinates::<CellT>::is_valid_coordinate(self, coord, dimensions) {
            let grid_2d_coord: Cartesian2DCoordinate = coord.as_cartesian_2d();
            dimensions.row_length(None) // all rows are the same length
                      .map(|length| {
                        let RowLength(row_size) = length;
                           (grid_2d_coord.y as usize * row_size) + grid_2d_coord.x as usize
                      })
        } else {
            None
        }
    }

    fn random_cell(&self, rng: &mut XorShiftRng,  dimensions: &Rc<GridDimensions>) -> CellT::Coord {

        let index = rng.gen::<usize>() % dimensions.size().0;
        CellT::Coord::from_row_major_index(index, dimensions.as_ref())
    }
}
