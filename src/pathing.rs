

// blah blah blah
// - Todo cell content printing
// def content_of(Cartesian2DCoordinate) for printing floodfill algorithm etc: ascii base36 letter (or base 64 maybe).
// Grid trait?

// Dijkstra's Algorithm
// For *each* cell we can generate floodfill data (matrix of steps to each other cell - hashtable really).
// Cartesian2DCoordinate: steps from from start mapping.
// Can be stored as a Vector of certain size.

// apply pathing algorithm to grid and get extra data structure info out of it...for every cell!

// How to map a Cartesian2DCoordinate to a vector location? It depends on the *dynamic* aspect of the Grid size and dimension.
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
use std::marker::PhantomData;
use std::ops::Add;

use itertools::Itertools;
use num::traits::{Bounded, One, Unsigned, Zero};
use smallvec::SmallVec;

use cells::{Cell, Coordinate};
use masks::BinaryMask2D;
use grid::{Grid, IndexType};
use grid_traits::GridIterators;
use units::{ColumnIndex, RowIndex};
use utils;
use utils::FnvHashMap;


// Trait (hack) used purely as a generic type parameter alias because it looks ugly to type this out each time
// Note generic parameter type aliases are not in the langauge.
// `type X = Y;` only works with concrete types.
pub trait MaxDistance
    : Zero + One + Bounded + Unsigned + Add + Debug + Clone + Copy + Display + LowerHex + Ord
    {
}
impl<T: Zero + One + Bounded + Unsigned + Add + Debug + Clone + Copy + Display + LowerHex + Ord> MaxDistance for T {}


#[derive(Debug, Clone)]
pub struct Distances<CellT: Cell, MaxDistanceT = u32> {
    start_coordinate: CellT::Coord,
    distances: FnvHashMap<CellT::Coord, MaxDistanceT>,
    max_distance: MaxDistanceT,
    cell_type: PhantomData<CellT>,
}

impl<CellT, MaxDistanceT> Distances<CellT, MaxDistanceT>
    where CellT: Cell,
          MaxDistanceT: MaxDistance
{
    pub fn new<GridIndexType, Iters>(grid: &Grid<GridIndexType, CellT, Iters>,
                                     start_coordinate: CellT::Coord)
                                     -> Option<Distances<CellT, MaxDistanceT>>
        where GridIndexType: IndexType,
              Iters: GridIterators<CellT>
    {

        if !grid.is_valid_coordinate(start_coordinate) {
            return None;
        }

        let mut max = Zero::zero();
        let cells_count = grid.size();
        let mut distances = utils::fnv_hashmap(cells_count);
        distances.insert(start_coordinate, Zero::zero());

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

                // All cells except the start cell are by default infinity distance from
                // the start until we process them, which is represented as Option::None when accessing the map.
                let distance_to_cell: MaxDistanceT = *distances.entry(*cell_coord)
                    .or_insert_with(Bounded::max_value);
                if distance_to_cell > max {
                    max = distance_to_cell;
                }

                let links: CellT::CoordinateSmallVec = grid.links(*cell_coord)
                    .expect("Source cell has an invalid cell coordinate.");
                for link_coordinate in &*links {

                    let distance_to_link: MaxDistanceT = *distances.entry(*link_coordinate)
                        .or_insert_with(Bounded::max_value);
                    if distance_to_link == Bounded::max_value() {

                        distances.insert(*link_coordinate, distance_to_cell + One::one());
                        new_frontier.push(*link_coordinate);
                    }
                }
            }
            frontier = new_frontier;
        }

        Some(Distances {
            start_coordinate: start_coordinate,
            distances: distances,
            max_distance: max,
            cell_type: PhantomData,
        })
    }

    #[inline(always)]
    pub fn start(&self) -> CellT::Coord {
        self.start_coordinate
    }

    #[inline(always)]
    pub fn max(&self) -> MaxDistanceT {
        self.max_distance
    }

    #[inline(always)]
    pub fn distances(&self) -> &FnvHashMap<CellT::Coord, MaxDistanceT> {
        &self.distances
    }

    #[inline(always)]
    pub fn distance_from_start_to(&self, coord: CellT::Coord) -> Option<MaxDistanceT> {
        self.distances.get(&coord).cloned()
    }

    pub fn furthest_points_on_grid(&self) -> SmallVec<[CellT::Coord; 8]> {
        let mut furthest = SmallVec::<[CellT::Coord; 8]>::new();
        let furthest_distance = self.max();

        for (coord, distance) in self.distances.iter() {
            if *distance == furthest_distance {
                furthest.push(*coord);
            }
        }
        furthest
    }
}

pub fn shortest_path<GridIndexType, MaxDistanceT, CellT, Iters>(grid: &Grid<GridIndexType, CellT, Iters>,
                                                         distances_from_start: &Distances<CellT, MaxDistanceT>,
                                                         end_point: CellT::Coord) -> Option<Vec<CellT::Coord>>
    where GridIndexType: IndexType,
          MaxDistanceT: MaxDistance,
          CellT: Cell,
          Iters: GridIterators<CellT>
{

    if let None = distances_from_start.distance_from_start_to(end_point) {
        // The end point is not reachable from start.
        return None;
    }

    let mut path = vec![end_point];
    let start = distances_from_start.start();
    let mut current_coord = end_point;

    while current_coord != start {

        let current_distance_to_start = distances_from_start.distance_from_start_to(current_coord)
            .expect("Coordinate invalid for distances_from_start data.");

        let linked_neighbours = grid.neighbours(current_coord)
            .iter()
            .cloned()
            .filter(|neighbour_coord| grid.is_linked(*neighbour_coord, current_coord))
            .collect::<CellT::CoordinateSmallVec>();
        let neighbour_distances = &linked_neighbours.iter()
            .map(|coord| {
                (*coord,
                 distances_from_start.distance_from_start_to(*coord)
                    .expect("Coordinate invalid for distances_from_start data."))
            })
            .collect::<SmallVec<[(CellT::Coord, MaxDistanceT); 8]>>();
        let closest_to_start: &Option<(CellT::Coord, MaxDistanceT)> = &neighbour_distances.iter()
            .cloned()
            .fold1(|closest_accumulator: (CellT::Coord, MaxDistanceT),
                    closest_candidate: (CellT::Coord, MaxDistanceT)| {
                if closest_candidate.1 < closest_accumulator.1 {
                    closest_candidate
                } else {
                    closest_accumulator
                }
            });

        if let Some((closer_coord, closer_distance)) = *closest_to_start {

            if closer_distance == current_distance_to_start {
                // We have not got any closer to the final goal, so there is no path there.
                return None;
            }

            current_coord = closer_coord;
            path.push(current_coord);

        } else {
            // There are no linked neighbours - this input data is broken.
            return None;
        }

    }

    path.reverse();
    Some(path)
}

/// Works only as long as we are looking at a perfect maze, otherwise you get back some arbitrary path back.
/// If the mask creates disconnected subgraphs it may not be the longest path.
pub fn dijkstra_longest_path<GridIndexType, MaxDistanceT, CellT, Iters>
    (grid: &Grid<GridIndexType, CellT, Iters>,
     mask: Option<&BinaryMask2D>)
     -> Option<Vec<CellT::Coord>>
    where GridIndexType: IndexType,
          MaxDistanceT: MaxDistance,
          CellT: Cell,
          Iters: GridIterators<CellT>
{
    // Distances to everywhere from an arbitrary start coordinate
    let arbitrary_start_point = if let Some(m) = mask {
        m.first_unmasked_coordinate()
    } else {
        Some(CellT::Coord::from_row_column_indices(ColumnIndex(0), RowIndex(0)))
    };

    if arbitrary_start_point.is_none() {
        return None;
    }

    let first_distances = Distances::<CellT, MaxDistanceT>::new(grid,
                                                                arbitrary_start_point.unwrap())
        .expect("Invalid start coordinate.");

    // The start of the longest path is just the point furthest away from an arbitrary initial point
    let long_path_start_coordinate = first_distances.furthest_points_on_grid()[0];

    let distances_from_start =
        Distances::<CellT, MaxDistanceT>::new(grid, long_path_start_coordinate).unwrap();
    let end_point = distances_from_start.furthest_points_on_grid()[0];

    shortest_path(&grid, &distances_from_start, end_point)
}


#[cfg(test)]
mod tests {

    use std::rc::Rc;
    use std::u32;

    use quickcheck::quickcheck;

    use super::*;
    use cells::{Cartesian2DCoordinate, SquareCell, Cell};
    use grid::Grid;
    use grid_dimensions::RectGridDimensions;
    use grid_coordinates::RectGridCoordinates;
    use grid_iterators::RectGridIterators;
    use units;


    /// A Small Rectangular Grid
    type SmallGrid = Grid<u8, SquareCell, RectGridIterators>;
    fn small_grid(width_and_height: usize) -> SmallGrid {
        SmallGrid::new(Rc::new(RectGridDimensions::new(units::RowLength(width_and_height), units::ColumnLength(width_and_height))),
                       Box::new(RectGridCoordinates),
                       RectGridIterators)
    }
    /// Distances between cells in a rectangular grid
    type SmallDistances = Distances<SquareCell, u8>;
    fn small_distances(g: &SmallGrid, coord: <SquareCell as Cell>::Coord) -> Option<SmallDistances> {
        SmallDistances::new(&g, coord)
    }

    static OUT_OF_GRID_COORDINATE: Cartesian2DCoordinate = Cartesian2DCoordinate {
        x: u32::MAX,
        y: u32::MAX,
    };

    #[test]
    fn distances_construction_requires_valid_start_coordinate() {
        let g = small_grid(3);
        let distances = small_distances(&g, OUT_OF_GRID_COORDINATE);
        assert!(distances.is_none());
    }

    #[test]
    fn start() {
        let g = small_grid(3);
        let start_coordinate = Cartesian2DCoordinate::new(1, 1);
        let distances = small_distances(&g, start_coordinate).unwrap();
        assert_eq!(start_coordinate, distances.start());
    }

    #[test]
    fn distances_to_unreachable_cells_is_none() {
        let g = small_grid(3);
        let start_coordinate = Cartesian2DCoordinate::new(0, 0);
        let distances = small_distances(&g, start_coordinate).unwrap();
        for coord in g.iter() {
            let d = distances.distance_from_start_to(coord);

            if coord != start_coordinate {
                assert!(d.is_none());
            } else {
                assert!(d.is_some());
                assert_eq!(d.unwrap(), 0);
            }
        }
    }

    #[test]
    fn distance_to_invalid_coordinate_is_none() {
        let g = small_grid(3);
        let start_coordinate = Cartesian2DCoordinate::new(0, 0);
        let distances = small_distances(&g, start_coordinate).unwrap();
        assert_eq!(distances.distance_from_start_to(OUT_OF_GRID_COORDINATE),
                   None);
    }

    #[test]
    fn distances_on_open_grid() {
        let mut g = small_grid(2);
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);
        let top_left = gc(0, 0);
        let top_right = gc(1, 0);
        let bottom_left = gc(0, 1);
        let bottom_right = gc(1, 1);
        g.link(top_left, top_right).expect("Link Failed");
        g.link(top_left, bottom_left).expect("Link Failed");
        g.link(top_right, bottom_right).expect("Link Failed");
        g.link(bottom_left, bottom_right).expect("Link Failed");

        let start_coordinate = gc(0, 0);
        let distances = small_distances(&g, start_coordinate).unwrap();

        assert_eq!(distances.distance_from_start_to(top_left), Some(0));
        assert_eq!(distances.distance_from_start_to(top_right), Some(1));
        assert_eq!(distances.distance_from_start_to(bottom_left), Some(1));
        assert_eq!(distances.distance_from_start_to(bottom_right), Some(2));
    }

    #[test]
    fn max_distance() {
        let mut g = small_grid(2);
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);
        let top_left = gc(0, 0);
        let top_right = gc(1, 0);
        let bottom_left = gc(0, 1);
        let bottom_right = gc(1, 1);
        g.link(top_left, top_right).expect("Link Failed");
        g.link(top_left, bottom_left).expect("Link Failed");
        g.link(top_right, bottom_right).expect("Link Failed");
        g.link(bottom_left, bottom_right).expect("Link Failed");
        let start_coordinate = gc(0, 0);
        let distances = small_distances(&g, start_coordinate).unwrap();
        assert_eq!(distances.max(), 2);
    }

    #[test]
    fn quickcheck_experiment() {

        fn p(_: Vec<isize>) -> bool {
            true
        }
        quickcheck(p as fn(Vec<isize>) -> bool)
    }
}
