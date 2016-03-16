

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

use num::traits::{FromPrimitive, Unsigned, Zero};
use petgraph::graph::IndexType;

use squaregrid::{GridCoordinate, SquareGrid};


struct DijkstraDistances<'a, GridIndexType: IndexType, MaxDistanceT = u32>
    where MaxDistanceT: FromPrimitive + Zero + Unsigned
{
    grid: &'a SquareGrid<GridIndexType>,
    start_coordinate: GridCoordinate,
    distances: Vec<MaxDistanceT>,
}

impl<'a, GridIndexType: IndexType, MaxDistanceT> DijkstraDistances<'a, GridIndexType, MaxDistanceT>
    where MaxDistanceT: FromPrimitive + Zero + Unsigned
{
    pub fn new(grid: &'a SquareGrid<GridIndexType>,
               start_coordinate: &GridCoordinate)
               -> DijkstraDistances<'a, GridIndexType, MaxDistanceT> {

        // All cells are by default 0 zero distance from the start until we process the grid.
        let cells_count = grid.size();
        let mut distances = Vec::with_capacity(cells_count);
        for _ in 0..cells_count {
            distances.push(Zero::zero());
        }

        DijkstraDistances {
            grid: grid,
            start_coordinate: start_coordinate.clone(),
            distances: distances,
        }
    }
}
