

use cells::SquareCell;
use grid::Grid;
use grid_coordinates::RectGridCoordinates;
use grid_dimensions::RectGridDimensions;
use grid_iterators::RectGridIterators;
use std::{u16, u32, u8};
use std::rc::Rc;
use units::{ColumnLength, RowLength};

pub type SmallRectangularGrid = Grid<u8, SquareCell, RectGridIterators>;
pub type MediumRectangularGrid = Grid<u16, SquareCell, RectGridIterators>;
pub type LargeRectangularGrid = Grid<u32, SquareCell, RectGridIterators>;

pub fn small_rect_grid(row_width: RowLength,
                       column_height: ColumnLength)
                       -> Option<SmallRectangularGrid> {

    if row_width.0 * column_height.0 <= u8::MAX as usize {

        Some(SmallRectangularGrid::new(Rc::new(RectGridDimensions::new(row_width, column_height)),
                                       Box::new(RectGridCoordinates),
                                       RectGridIterators))
    } else {
        None
    }
}

pub fn medium_rect_grid(row_width: RowLength,
                        column_height: ColumnLength)
                        -> Option<MediumRectangularGrid> {

    if row_width.0 * column_height.0 <= u16::MAX as usize {

        Some(MediumRectangularGrid::new(Rc::new(RectGridDimensions::new(row_width, column_height)),
                                        Box::new(RectGridCoordinates),
                                        RectGridIterators))
    } else {
        None
    }
}

pub fn large_rect_grid(row_width: RowLength,
                       column_height: ColumnLength)
                       -> Option<LargeRectangularGrid> {

    if row_width.0 * column_height.0 <= u32::MAX as usize {

        Some(LargeRectangularGrid::new(Rc::new(RectGridDimensions::new(row_width, column_height)),
                                       Box::new(RectGridCoordinates),
                                       RectGridIterators))
    } else {
        None
    }
}
