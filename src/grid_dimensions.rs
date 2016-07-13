use std::rc::Rc;

use cells::{Cartesian2DCoordinate, Cell, Coordinate};
use grid_traits::{GridDimensions};
use units::RowLength;


#[derive(Debug, Copy, Clone)]
pub struct RectGridDimensions {

}

impl RectGridDimensions {
    pub fn new() -> RectGridDimensions {
        RectGridDimensions {

        }
    }
}

impl GridDimensions for RectGridDimensions {


}
