use std::marker::PhantomData;
use std::fmt;

use cells::Cell;
use grid_traits::{GridDimensions, GridIterators};


pub struct RectGridIterators;
impl<'a, CellT: Cell> GridIterators<CellT> for RectGridIterators {
    type CellIter = RectGridCellIter<'a, CellT>;
    type BatchIter = RectBatchIter;

    fn iter(&self, dimensions: &GridDimensions) -> Self::CellIter {
        RectGridCellIter {
            dimensions: dimensions,
            current_cell_number: 0,
            cells_count: dimensions.size().0,
            cell_type: PhantomData
        }
    }

    fn iter_row(&self, dimensions: &GridDimensions) -> Self::BatchIter {
        RectBatchIter {}
    }

    fn iter_column(&self, dimensions: &GridDimensions) -> Self::BatchIter {
        RectBatchIter {}
    }
}

#[derive(Copy, Clone)]
pub struct RectGridCellIter<'a, CellT: Cell> {
    dimensions: &'a GridDimensions,
    current_cell_number: usize,
    cells_count: usize,
    cell_type: PhantomData<CellT>
}
impl<'a, CellT: Cell> fmt::Debug for RectGridCellIter<'a, CellT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CellIter :: current_cell_number: {:?}, cells_count: {:?}",
               self.current_cell_number, self.cells_count)
    }
}
impl<'a, CellT: Cell> ExactSizeIterator for RectGridCellIter<'a, CellT> { } // default impl using size_hint()
impl<'a, CellT: Cell> Iterator for RectGridCellIter<'a, CellT> {
    type Item = CellT::Coord;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_cell_number < self.cells_count {
            let coord = Self::Item::from_row_major_index(self.current_cell_number,
                                                         self.dimensions);
            self.current_cell_number += 1;
            Some(coord)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower_bound = self.cells_count - self.current_cell_number;
        let upper_bound = lower_bound;
        (lower_bound, Some(upper_bound))
    }
}

struct RectBatchIter {

}
