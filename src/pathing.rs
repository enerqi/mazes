

// blah blah blah
// - Todo cell content printing
// def content_of(gridcoordinate) for printing floodfill algorithm etc: ascii base36 letter (or base 64 maybe).
// Grid trait?

// Dijkstra's Algorithm
// For *each* cell we can generate floodfill data (matrix of steps to each other cell - hashtable really).
// GridCoordinate: steps from from start mapping.
// Can be stored as a Vector of certain size.

// apply pathing algorithm to grid and get extra data structure info out of it...for every cell!

// How to map a gridcoordinate to a vector location? It depends on the *dynamic* aspect of the Grid size and dimension.
// So it's not a compile time decision.
// How to safely associate with only 1 live grid?
// Well OO would make is a subclass with new data and a bloated base interface.
// - The pathing data needs to record the size/dimension at initialisation time
//   x this has no guarantee to be calculated for the relevant grid
// OR
// - The pathing data needs to be initialised with a reference to the grid processed
//   x this is an immutable borrow, which would perhaps inconveniently prevent other mutable updates to the grid
//   ✓ any mutation of the grid (most Grid functions are immutable) could invalidate the pathing data
//   ? we could always copy the pathing data into a new data structure without the &Grid
//     In this case we almost want persistent data structures, old versions around - perhaps of the pathing data and the graph
// OR
// - Weak (requires downgrading an RC<T>) pointer or RC<T>
//   x requires heap allocating the graph, though that's much data - most of it is implemented as Vectors anyway.



use std::fmt::{Debug, Display, LowerHex};
use std::ops::Add;

use num::traits::{Bounded, One, Unsigned, Zero};
use petgraph::graph::IndexType;

use squaregrid::{GridCoordinate, GridDisplay, SquareGrid};

#[derive(Debug, Clone)]
pub struct DijkstraDistances<'a, GridIndexType: IndexType, MaxDistanceT=GridIndexType>
    where MaxDistanceT: Zero + One + Bounded + Unsigned + Add + Debug + Clone + Copy + Display + LowerHex
{
    grid: &'a SquareGrid<'a, GridIndexType>,
    start_coordinate: GridCoordinate,
    distances: Vec<MaxDistanceT>, /* This could be a vec_map, but all the keys should always be used so not really worth it. */
}

impl<'a, GridIndexType: IndexType, MaxDistanceT> DijkstraDistances<'a, GridIndexType, MaxDistanceT>
    where MaxDistanceT: Zero + One + Bounded + Unsigned + Add + Debug + Clone + Copy + Display + LowerHex
{
    pub fn new(grid: &'a SquareGrid<GridIndexType>,
               start_coordinate: GridCoordinate)
               -> Option<DijkstraDistances<'a, GridIndexType, MaxDistanceT>> {

        let start_coord_index_opt = grid.grid_coordinate_to_index(start_coordinate);
        if let None = start_coord_index_opt {
            // Invalid start coordinate
            return None;
        }
        let start_coord_index = start_coord_index_opt.unwrap();

        // All cells except the start cell are by default (infinity) max_value distance from the start until we process the grid.
        let cells_count = grid.size();
        let mut distances: Vec<MaxDistanceT> = vec![Bounded::max_value(); cells_count];
        distances[start_coord_index] = Zero::zero();

        // Wonder how this compares with standard Dijkstra shortest path tree algorithm...
        // We don't have any weights on the edges/links to consider, every step is just one from the previous cell
        // so we never have to change the distance to a cell if it has been updated once in the distances vec - the shortest
        // distance has already been set for that cell.
        //
        // The frontier vec does not need to be a set datastructure as the distances vec effectively tracks whether a cell
        // already been processed - acts as a visited set aswell as a storer of the floodfill distances.
        let mut frontier = vec![start_coordinate];
        while !frontier.is_empty() {

            let mut new_frontier = vec![];
            for cell_coord in &frontier {

                let cell_index = grid.grid_coordinate_to_index(*cell_coord)
                                     .expect("Frontier cell has an invalid cell coordinate");
                let distance_to_cell: MaxDistanceT = distances[cell_index].clone();

                let links = grid.links(*cell_coord)
                                .expect("Source cell has an invalid cell coordinate.");
                for link in &*links {

                    let gc_index = grid.grid_coordinate_to_index(*link)
                                       .expect("Linked cell has an invalid cell coordinate");
                    if distances[gc_index] == Bounded::max_value() {

                        distances[gc_index] = distance_to_cell + One::one();
                        new_frontier.push(*link);
                    }
                }
            }
            frontier = new_frontier;
        }

        Some(DijkstraDistances {
             grid: grid,
             start_coordinate: start_coordinate,
             distances: distances})
    }

    pub fn start(&self) -> GridCoordinate {
        self.start_coordinate
    }

    pub fn distance_from_start_to(&self, coord: GridCoordinate) -> Option<MaxDistanceT> {
        self.grid.grid_coordinate_to_index(coord).map(|index| self.distances[index])
    }
}

impl<'a, GridIndexType: IndexType, MaxDistanceT> GridDisplay for DijkstraDistances<'a, GridIndexType, MaxDistanceT>
    where MaxDistanceT: Zero + One + Bounded + Unsigned + Add + Debug + Clone + Copy + Display + LowerHex
{
    fn render_cell_body(&self, coord: GridCoordinate) -> String {

        let index = self.grid.grid_coordinate_to_index(coord).expect("An invalid GridCoordinate is being Displayed.");
        let distance = self.distances[index];
        // centre align, padding 3, lowercase hexadecimal
        format!("{:^3x}", distance)
    }
}

#[cfg(test)]
mod tests {

    //use itertools::Itertools;
    use std::{u8, u32};

    use super::*;
    use squaregrid::{GridCoordinate, SquareGrid};

    type SmallGrid<'a> = SquareGrid<'a, u8>;
    type SmallDistances<'a> = DijkstraDistances<'a, u8, u8>;

    static OUT_OF_GRID_COORDINATE: GridCoordinate = GridCoordinate{x: u32::MAX, y: u32::MAX};

    #[test]
    fn distances_construction_requires_valid_start_coordinate() {
        let g = SmallGrid::new(2);
        let distances = SmallDistances::new(&g, OUT_OF_GRID_COORDINATE);
        assert!(distances.is_none());
    }

    #[test]
    fn start() {
        let g = SmallGrid::new(3);
        let start_coordinate = GridCoordinate::new(1,1);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();
        assert_eq!(start_coordinate, distances.start());
    }

    #[test]
    fn distances_to_unreachable_cells() {
        let g = SmallGrid::new(3);
        let start_coordinate = GridCoordinate::new(0,0);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();
        for coord in g.iter() {
            let d = distances.distance_from_start_to(coord);

            if coord != start_coordinate {
                assert!(d.is_some());
                assert_eq!(d.unwrap(), u8::MAX);
            } else {
                assert!(d.is_some());
                assert_eq!(d.unwrap(), 0);
            }
        }
    }

    #[test]
    fn distance_to_invalid_coordinate_is_none() {
        let g = SmallGrid::new(3);
        let start_coordinate = GridCoordinate::new(0,0);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();
        assert_eq!(distances.distance_from_start_to(OUT_OF_GRID_COORDINATE), None);
    }

    #[test]
    fn distances_on_open_grid() {
        let mut g = SmallGrid::new(2);
        let gc =|x, y| GridCoordinate::new(x, y);
        let top_left = gc(0,0);
        let top_right = gc(1,0);
        let bottom_left = gc(0,1);
        let bottom_right = gc(1,1);
        g.link(top_left, top_right).expect("Link Failed");
        g.link(top_left, bottom_left).expect("Link Failed");
        g.link(top_right, bottom_right).expect("Link Failed");
        g.link(bottom_left, bottom_right).expect("Link Failed");

        let start_coordinate = gc(0,0);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();

        assert_eq!(distances.distance_from_start_to(top_left), Some(0));
        assert_eq!(distances.distance_from_start_to(top_right), Some(1));
        assert_eq!(distances.distance_from_start_to(bottom_left), Some(1));
        assert_eq!(distances.distance_from_start_to(bottom_right), Some(2));
    }

}
