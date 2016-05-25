use std::convert::From;

//use smallvec::SmallVec;


pub trait Coordinate {

    // type CoordinateSmallVec;
    // type CoordinateOptionSmallVec;

    fn as_cartesian_2d(&self) -> Cartesian2DCoordinate;
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug, Ord, PartialOrd)]
pub struct Cartesian2DCoordinate {
    pub x: u32,
    pub y: u32,
}
impl Cartesian2DCoordinate {
    pub fn new(x: u32, y: u32) -> Cartesian2DCoordinate {
        Cartesian2DCoordinate { x: x, y: y }
    }
}
impl Coordinate for Cartesian2DCoordinate {

    // type CoordinateSmallVec = SmallVec<[Cartesian2DCoordinate; 4]>;
    // type CoordinateOptionSmallVec = SmallVec<[Option<Cartesian2DCoordinate>; 4]>;

    fn as_cartesian_2d(&self) -> Cartesian2DCoordinate {
        self.clone()
    }
}
impl From<(u32, u32)> for Cartesian2DCoordinate {
    fn from(x_y_pair: (u32, u32)) -> Cartesian2DCoordinate {
        Cartesian2DCoordinate::new(x_y_pair.0, x_y_pair.1)
    }
}
