

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

use itertools::Itertools;
use num::traits::{Bounded, One, Unsigned, Zero};
use smallvec::SmallVec;

use coordinates::GridCoordinate;
use masks::BinaryMask2D;
use squaregrid::{CoordinateSmallVec, GridDisplay, IndexType, SquareGrid};
use utils;
use utils::{FnvHashMap, FnvHashSet};


// Trait (hack) used purely as a generic type parameter alias because it looks ugly to type this out each time
// Note generic parameter type aliases are not in the langauge.
// `type X = Y;` only works with concrete types.
pub trait MaxDistance
    : Zero + One + Bounded + Unsigned + Add + Debug + Clone + Copy + Display + LowerHex + Ord
    {
}
impl<T: Zero + One + Bounded + Unsigned + Add + Debug + Clone + Copy + Display + LowerHex + Ord> MaxDistance for T {}


#[derive(Debug, Clone)]
pub struct DijkstraDistances<MaxDistanceT = u32> {
    start_coordinate: GridCoordinate,
    distances: FnvHashMap<GridCoordinate, MaxDistanceT>,
    max_distance: MaxDistanceT,
}

impl<MaxDistanceT> DijkstraDistances<MaxDistanceT>
    where MaxDistanceT: MaxDistance
{
    pub fn new<GridIndexType: IndexType>(grid: &SquareGrid<GridIndexType>,
                                         start_coordinate: GridCoordinate)
                                         -> Option<DijkstraDistances<MaxDistanceT>> {

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

                let links = grid.links(*cell_coord)
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

        Some(DijkstraDistances {
            start_coordinate: start_coordinate,
            distances: distances,
            max_distance: max,
        })
    }

    pub fn start(&self) -> GridCoordinate {
        self.start_coordinate
    }

    pub fn max(&self) -> MaxDistanceT {
        self.max_distance
    }

    pub fn distance_from_start_to(&self, coord: GridCoordinate) -> Option<MaxDistanceT> {
        self.distances.get(&coord).cloned()
    }

    pub fn furthest_points_on_grid(&self) -> CoordinateSmallVec {
        let mut furthest = CoordinateSmallVec::new();
        let furthest_distance = self.max();

        for (coord, distance) in self.distances.iter() {
            if *distance == furthest_distance {
                furthest.push(*coord);
            }
        }
        furthest
    }
}

impl<MaxDistanceT> GridDisplay for DijkstraDistances<MaxDistanceT>
    where MaxDistanceT: MaxDistance
{
    fn render_cell_body(&self, coord: GridCoordinate) -> String {

        // In case DijkstraDistances is used with a different grid check for Vec access being in bounds.
        // N.B.
        // Keeping a reference to the grid that was processed is possible, but the circular nature of distances to Grid
        // and Grid to (distances as GridDisplay) means we need Rc and Weak pointers, in particular Rc<RefCell<_>> for the
        // maze so that we could mutate it to inject the (distance as GridDisplay) and the (distance as GridDisplay) could be
        // given an Rc<_> downgraded to Weak<_> to refer to the Grid...or maybe GridDisplay holds a &SquareGrid but that won't
        // work as the lifetime of any Rc is unknown and &SquareGrid would need a 'static lifetime.
        // As the ref from the (distance as GridDisplay) to SquareGrid is not &T and the Rc<RefCell> avoids static borrow check
        // rules there are no guarantees that the graph on the SquareGrid cannot change after distances has been created.
        //
        // *Iff* a DijkstraDistances were always to be created with every SquareGrid, such that the lifetimes are the same
        // the SquareGrid could have a RefCell<Option<&GridDisplay>> and the GridDisplay could have &SquareGrid which would
        // freeze as immutable the graph of the SquareGrid.

        if let Some(d) = self.distances.get(&coord) {
            // centre align, padding 3, lowercase hexadecimal
            format!("{:^3x}", d)
        } else {
            String::from("   ")
        }
    }
}

#[derive(Debug)]
pub struct StartEndPointsDisplay {
    start_coordinates: CoordinateSmallVec,
    end_coordinates: CoordinateSmallVec,
}
impl StartEndPointsDisplay {
    pub fn new(starts: CoordinateSmallVec, ends: CoordinateSmallVec) -> StartEndPointsDisplay {
        StartEndPointsDisplay {
            start_coordinates: starts,
            end_coordinates: ends,
        }
    }
}
impl GridDisplay for StartEndPointsDisplay {
    fn render_cell_body(&self, coord: GridCoordinate) -> String {

        let contains_coordinate =
            |coordinates: &CoordinateSmallVec| coordinates.iter().any(|&c| c == coord);

        if contains_coordinate(&self.start_coordinates) {
            String::from(" S ")

        } else if contains_coordinate(&self.end_coordinates) {

            String::from(" E ")

        } else {
            String::from("   ")
        }
    }
}

#[derive(Debug)]
pub struct PathDisplay {
    on_path_coordinates: FnvHashSet<GridCoordinate>,
}
impl PathDisplay {
    pub fn new(path: &[GridCoordinate]) -> Self {
        PathDisplay { on_path_coordinates: path.iter().cloned().collect() }
    }
}
impl GridDisplay for PathDisplay {
    fn render_cell_body(&self, coord: GridCoordinate) -> String {
        if self.on_path_coordinates.contains(&coord) {
            String::from(" . ")
        } else {
            String::from("   ")
        }
    }
}

pub fn shortest_path<GridIndexType, MaxDistanceT>(grid: &SquareGrid<GridIndexType>,
                                                  distances_from_start: &DijkstraDistances<MaxDistanceT>,
                                                  end_point: GridCoordinate) -> Option<Vec<GridCoordinate>>
    where GridIndexType: IndexType, MaxDistanceT: MaxDistance
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

        let mut linked_neighbours = grid.neighbours(current_coord)
            .iter()
            .cloned()
            .filter(|neighbour_coord| grid.is_linked(*neighbour_coord, current_coord))
            .collect::<CoordinateSmallVec>();
        let mut neighbour_distances = linked_neighbours.into_iter()
            .map(|coord| {
                (coord,
                 distances_from_start.distance_from_start_to(coord)
                    .expect("Coordinate invalid for distances_from_start data."))
            })
            .collect::<SmallVec<[(GridCoordinate, MaxDistanceT); 4]>>();
        let closest_to_start = neighbour_distances.into_iter()
            .fold1(|closest_accumulator, closest_candidate| {
                if closest_candidate.1 < closest_accumulator.1 {
                    closest_candidate
                } else {
                    closest_accumulator
                }
            });

        if let Some((closer_coord, closer_distance)) = closest_to_start {

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
pub fn dijkstra_longest_path<GridIndexType, MaxDistanceT>(grid: &SquareGrid<GridIndexType>,
                                                          mask: Option<&BinaryMask2D>)
                                                          -> Option<Vec<GridCoordinate>>
    where GridIndexType: IndexType,
          MaxDistanceT: MaxDistance
{
    // Distances to everywhere from an arbitrary start coordinate
    let arbitrary_start_point = if let Some(m) = mask {
        m.first_unmasked_coordinate()
    } else {
        Some(GridCoordinate::new(0, 0))
    };

    if arbitrary_start_point.is_none() {
        return None;
    }

    let first_distances = DijkstraDistances::<MaxDistanceT>::new(grid,
                                                                 arbitrary_start_point.unwrap())
        .expect("Invalid start coordinate.");

    // The start of the longest path is just the point furthest away from an arbitrary initial point
    let long_path_start_coordinate = first_distances.furthest_points_on_grid()[0];

    let distances_from_start =
        DijkstraDistances::<MaxDistanceT>::new(grid, long_path_start_coordinate).unwrap();
    let end_point = distances_from_start.furthest_points_on_grid()[0];

    shortest_path(&grid, &distances_from_start, end_point)
}


#[cfg(test)]
mod tests {

    use std::u32;
    use quickcheck::quickcheck;

    use super::*;
    use coordinates::GridCoordinate;
    use squaregrid::SquareGrid;

    type SmallGrid = SquareGrid<u8>;
    type SmallDistances = DijkstraDistances<u8>;

    static OUT_OF_GRID_COORDINATE: GridCoordinate = GridCoordinate {
        x: u32::MAX,
        y: u32::MAX,
    };

    #[test]
    fn distances_construction_requires_valid_start_coordinate() {
        let g = SmallGrid::new(3);
        let distances = SmallDistances::new(&g, OUT_OF_GRID_COORDINATE);
        assert!(distances.is_none());
    }

    #[test]
    fn start() {
        let g = SmallGrid::new(3);
        let start_coordinate = GridCoordinate::new(1, 1);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();
        assert_eq!(start_coordinate, distances.start());
    }

    #[test]
    fn distances_to_unreachable_cells_is_none() {
        let g = SmallGrid::new(3);
        let start_coordinate = GridCoordinate::new(0, 0);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();
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
        let g = SmallGrid::new(3);
        let start_coordinate = GridCoordinate::new(0, 0);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();
        assert_eq!(distances.distance_from_start_to(OUT_OF_GRID_COORDINATE),
                   None);
    }

    #[test]
    fn distances_on_open_grid() {
        let mut g = SmallGrid::new(2);
        let gc = |x, y| GridCoordinate::new(x, y);
        let top_left = gc(0, 0);
        let top_right = gc(1, 0);
        let bottom_left = gc(0, 1);
        let bottom_right = gc(1, 1);
        g.link(top_left, top_right).expect("Link Failed");
        g.link(top_left, bottom_left).expect("Link Failed");
        g.link(top_right, bottom_right).expect("Link Failed");
        g.link(bottom_left, bottom_right).expect("Link Failed");

        let start_coordinate = gc(0, 0);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();

        assert_eq!(distances.distance_from_start_to(top_left), Some(0));
        assert_eq!(distances.distance_from_start_to(top_right), Some(1));
        assert_eq!(distances.distance_from_start_to(bottom_left), Some(1));
        assert_eq!(distances.distance_from_start_to(bottom_right), Some(2));
    }

    #[test]
    fn max_distance() {
        let mut g = SmallGrid::new(2);
        let gc = |x, y| GridCoordinate::new(x, y);
        let top_left = gc(0, 0);
        let top_right = gc(1, 0);
        let bottom_left = gc(0, 1);
        let bottom_right = gc(1, 1);
        g.link(top_left, top_right).expect("Link Failed");
        g.link(top_left, bottom_left).expect("Link Failed");
        g.link(top_right, bottom_right).expect("Link Failed");
        g.link(bottom_left, bottom_right).expect("Link Failed");
        let start_coordinate = gc(0, 0);
        let distances = SmallDistances::new(&g, start_coordinate).unwrap();
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
