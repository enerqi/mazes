use crate::{
    cells::{Cartesian2DCoordinate, Cell, Coordinate},
    grid_traits::{GridCoordinates, GridDimensions},
    units::{NodesCount, RowIndex, RowLength},
};

use rand::{rngs::SmallRng, Rng};
use std::rc::Rc;

#[derive(Debug, Copy, Clone)]
pub struct RectGridCoordinates;

impl<CellT: Cell> GridCoordinates<CellT> for RectGridCoordinates {
    fn grid_coordinate_to_index(&self, coord: CellT::Coord, dimensions: &Rc<dyn GridDimensions>) -> Option<usize> {
        if GridCoordinates::<CellT>::is_valid_coordinate(self, coord, dimensions) {
            let grid_2d_coord: Cartesian2DCoordinate = coord.as_cartesian_2d();
            dimensions
                .row_length(None) // all rows are the same length
                .map(|length| {
                    let RowLength(row_size) = length;
                    (grid_2d_coord.y as usize * row_size) + grid_2d_coord.x as usize
                })
        } else {
            None
        }
    }

    fn random_cell(&self, rng: &mut SmallRng, dimensions: &Rc<dyn GridDimensions>) -> CellT::Coord {
        let index = rng.gen::<usize>() % dimensions.size().0;
        CellT::Coord::from_row_major_index(index, dimensions.as_ref())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PolarGridCoordinates;

impl<CellT: Cell> GridCoordinates<CellT> for PolarGridCoordinates {
    fn grid_coordinate_to_index(&self, coord: CellT::Coord, dimensions: &Rc<dyn GridDimensions>) -> Option<usize> {
        if GridCoordinates::<CellT>::is_valid_coordinate(self, coord, dimensions) {
            // Transform coordinate to neutral format
            let grid_2d_coord: Cartesian2DCoordinate = coord.as_cartesian_2d();

            let row_num = grid_2d_coord.y;

            dimensions
                .nodes_count_up_to(RowIndex(row_num as usize))
                .map(|node_count| {
                    let NodesCount(count) = node_count;
                    grid_2d_coord.x as usize + count
                })
        } else {
            None
        }
    }

    fn random_cell(&self, rng: &mut SmallRng, dimensions: &Rc<dyn GridDimensions>) -> CellT::Coord {
        let index = rng.gen::<usize>() % dimensions.size().0;
        CellT::Coord::from_row_major_index(index, dimensions.as_ref())
    }
}
