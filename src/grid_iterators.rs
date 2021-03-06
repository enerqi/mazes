use crate::{
    cells::{Cell, Coordinate},
    grid_traits::{GridDimensions, GridIterators},
    units::{ColumnIndex, ColumnLength, ColumnsCount, RowIndex, RowLength, RowsCount},
};
use std::{fmt, marker::PhantomData, rc::Rc};

#[derive(Debug, Copy, Clone)]
pub struct RectGridIterators;

impl<CellT: Cell> GridIterators<CellT> for RectGridIterators {
    type CellIter = RectGridCellIter<CellT>;
    type BatchIter = RectBatchIter<CellT>;

    fn iter(&self, dimensions: &Rc<dyn GridDimensions>) -> Self::CellIter {
        RectGridCellIter::<CellT> {
            dimensions: dimensions.clone(),
            current_cell_number: 0,
            cells_count: dimensions.size().0,
            cell_type: PhantomData,
        }
    }

    fn iter_row(&self, dimensions: &Rc<dyn GridDimensions>) -> Self::BatchIter {
        RectBatchIter::<CellT>::new(BatchIterType::Row, dimensions)
    }

    fn iter_column(&self, dimensions: &Rc<dyn GridDimensions>) -> Self::BatchIter {
        RectBatchIter::<CellT>::new(BatchIterType::Column, dimensions)
    }
}

#[derive(Clone)]
pub struct RectGridCellIter<CellT: Cell> {
    dimensions: Rc<dyn GridDimensions>,
    current_cell_number: usize,
    cells_count: usize,
    cell_type: PhantomData<CellT>,
}

impl<CellT: Cell> fmt::Debug for RectGridCellIter<CellT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CellIter :: current_cell_number: {:?}, cells_count: {:?}",
            self.current_cell_number, self.cells_count
        )
    }
}

impl<CellT: Cell> ExactSizeIterator for RectGridCellIter<CellT> {} // default impl using size_hint()
impl<CellT: Cell> Iterator for RectGridCellIter<CellT> {
    type Item = CellT::Coord;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_cell_number < self.cells_count {
            let coord = Self::Item::from_row_major_index(
                self.current_cell_number,
                self.dimensions.as_ref(),
            );
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

#[derive(Debug, Copy, Clone)]
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
    fn new(iter_type: BatchIterType, dimensions: &Rc<dyn GridDimensions>) -> RectBatchIter<CellT> {
        let rows_size = dimensions.rows();
        let cols_size = dimensions.columns();
        RectBatchIter {
            iter_type,
            iter_initial_length: rows_size.0 * cols_size.0,
            current_index: 0,
            row_length: dimensions.row_length(None).unwrap(),
            rows_size,
            col_length: dimensions.column_length(None),
            cols_size,
            cell_type: PhantomData,
        }
    }
}

impl<CellT: Cell> ExactSizeIterator for RectBatchIter<CellT> {} // default impl using size_hint()
impl<CellT: Cell> Iterator for RectBatchIter<CellT> {
    type Item = Vec<CellT::Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if let BatchIterType::Row = self.iter_type {
            let RowsCount(count) = self.rows_size;
            if self.current_index < count {
                let RowLength(length) = self.row_length;
                let coords = (0..length)
                    .map(|i: usize| {
                        CellT::Coord::from_row_column_indices(
                            ColumnIndex(i),
                            RowIndex(self.current_index),
                        )
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
                    .map(|i: usize| {
                        CellT::Coord::from_row_column_indices(
                            ColumnIndex(self.current_index),
                            RowIndex(i),
                        )
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

#[derive(Debug, Copy, Clone)]
pub struct PolarGridIterators;

impl<CellT: Cell> GridIterators<CellT> for PolarGridIterators {
    type CellIter = RectGridCellIter<CellT>; // exactly the same as RectGrid for the moment as they have same underlying coordinate type
    type BatchIter = PolarBatchIter<CellT>;

    fn iter(&self, dimensions: &Rc<dyn GridDimensions>) -> Self::CellIter {
        RectGridCellIter::<CellT> {
            dimensions: dimensions.clone(),
            current_cell_number: 0,
            cells_count: dimensions.size().0,
            cell_type: PhantomData,
        }
    }

    fn iter_row(&self, dimensions: &Rc<dyn GridDimensions>) -> Self::BatchIter {
        PolarBatchIter::<CellT>::new(BatchIterType::Row, dimensions)
    }

    fn iter_column(&self, dimensions: &Rc<dyn GridDimensions>) -> Self::BatchIter {
        PolarBatchIter::<CellT>::new(BatchIterType::Column, dimensions)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PolarBatchIter<CellT> {
    iter_type: BatchIterType,
    iter_initial_length: usize,
    current_index: usize,
    row_length: RowLength,
    rows_size: RowsCount,
    col_length: ColumnLength,
    cols_size: ColumnsCount,
    cell_type: PhantomData<CellT>,
}

impl<CellT> PolarBatchIter<CellT> {
    fn new(iter_type: BatchIterType, dimensions: &Rc<dyn GridDimensions>) -> PolarBatchIter<CellT> {
        let rows_size = dimensions.rows();
        let cols_size = dimensions.columns();
        PolarBatchIter {
            iter_type,
            iter_initial_length: rows_size.0 * cols_size.0,
            current_index: 0,
            row_length: dimensions.row_length(None).unwrap(),
            rows_size,
            col_length: dimensions.column_length(None),
            cols_size,
            cell_type: PhantomData,
        }
    }
}

impl<CellT: Cell> ExactSizeIterator for PolarBatchIter<CellT> {} // default impl using size_hint()
impl<CellT: Cell> Iterator for PolarBatchIter<CellT> {
    type Item = Vec<CellT::Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        // this will be really slow for by column, row is fine
        // start at the outside of the circle and work into the centre to define a "column"

        if let BatchIterType::Row = self.iter_type {
            None
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower_bound = self.iter_initial_length - self.current_index;
        let upper_bound = lower_bound;
        (lower_bound, Some(upper_bound))
    }
}
