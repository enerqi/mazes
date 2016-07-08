use std::marker::PhantomData;
use std::fmt;

use cells::Cell;
use grid_traits::{GridDimensions, GridIterators};
use units::{RowsCount, RowLength, RowIndex, ColumnsCount, ColumnLength,
            ColumnIndex, NodesCount, EdgesCount};

pub struct RectGridIterators<'a> {
    cell_iter_dimension: PhantomData<&'a GridDimensions>
}
impl<'a, CellT: Cell> GridIterators<CellT> for RectGridIterators<'a> {
    type CellIter = RectGridCellIter<'a, CellT>;
    type BatchIter = RectBatchIter<CellT>;

    fn iter(&self, dimensions: &'a GridDimensions) -> Self::CellIter {
        RectGridCellIter::<CellT> {
            dimensions: dimensions,
            current_cell_number: 0,
            cells_count: dimensions.size().0,
            cell_type: PhantomData
        }
    }

    fn iter_row(&self, dimensions: &GridDimensions) -> Self::BatchIter {
        RectBatchIter::<CellT>::new(BatchIterType::Row, dimensions)
    }

    fn iter_column(&self, dimensions: &GridDimensions) -> Self::BatchIter {
        RectBatchIter::<CellT>::new(BatchIterType::Column, dimensions)
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

#[derive(Debug, Copy, Clone)]
enum BatchIterType {
    Row,
    Column,
}

pub struct RectBatchIter<CellT> {
    iter_type: BatchIterType,
    iter_initial_length: usize,
    current_index: usize,
    row_length: RowLength,
    rows_size: RowsCount,
    col_length: ColumnLength,
    cols_size: ColumnsCount,
    cell_type: PhantomData<CellT>,
}

impl<CellT> RectBatchIter<CellT> {
    fn new(iter_type: BatchIterType, dimensions: &GridDimensions) -> RectBatchIter<CellT> {
        let rows_size = dimensions.rows();
        let cols_size = dimensions.columns();
        RectBatchIter {
            iter_type: BatchIterType::Row,
            iter_initial_length: rows_size * cols_size,
            current_index: 0,
            row_length: dimensions.row_length(None).unwrap(),
            rows_size: rows_size,
            col_length: dimensions.column_length(None),
            cols_size: cols_size,
        }
    }
}

impl<CellT: Cell> ExactSizeIterator for RectBatchIter<CellT> { } // default impl using size_hint()
impl<CellT: Cell> Iterator for RectBatchIter<CellT> {
    type Item = Vec<CellT::Coord>;
    fn next(&mut self) -> Option<Self::Item> {

        if let BatchIterType::Row = self.iter_type {

            let RowsCount(count) = self.rows_size;
            if self.current_index < count {
                let RowLength(length) = self.row_length;
                let coords = (0..length)
                    .into_iter()
                    .map(|i: usize| {
                        CellT::Coord::from_row_column_indices(ColumnIndex(i), RowIndex(self.current_index))
                    })
                    .collect();
                self.current_index += 1;
                Some(coords)
            } else {
                None
            }

        } else {

            let ColumnsCount(count) = self.cols_size;
            if self.current_index < count {
                let ColumnLength(length) = self.col_length;
                let coords = (0..length)
                    .into_iter()
                    .map(|i: usize| {
                        CellT::Coord::from_row_column_indices(ColumnIndex(self.current_index), RowIndex(i))
                    })
                    .collect();
                self.current_index += 1;
                Some(coords)
            } else {
                None
            }

        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower_bound = self.iter_initial_length - self.current_index;
        let upper_bound = lower_bound;
        (lower_bound, Some(upper_bound))
    }
}
