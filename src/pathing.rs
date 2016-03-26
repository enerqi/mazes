

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


use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hash};
use std::ops::Add;

use fnv::FnvHasher;
use num::traits::{One, Unsigned, Zero};
use petgraph::graph::IndexType;

use squaregrid::{GridCoordinate, SquareGrid};


struct DijkstraDistances<'a, GridIndexType: IndexType, MaxDistanceT = u32>
    where MaxDistanceT: Zero + One + Unsigned + Add + Clone + Copy
{
    grid: &'a SquareGrid<GridIndexType>,
    start_coordinate: GridCoordinate,
    distances: Vec<MaxDistanceT>, /* This could be a vec_map, but all the keys should always be used so not really worth it. */
}

impl<'a, GridIndexType: IndexType, MaxDistanceT> DijkstraDistances<'a, GridIndexType, MaxDistanceT>
    where MaxDistanceT: Zero + One + Unsigned + Add + Clone + Copy
{
    pub fn new(grid: &'a SquareGrid<GridIndexType>,
               start_coordinate: &GridCoordinate)
               -> DijkstraDistances<'a, GridIndexType, MaxDistanceT> {

        // All cells are by default 0 zero distance from the start until we process the grid.
        let cells_count = grid.size();
        let mut distances: Vec<MaxDistanceT> = Vec::with_capacity(cells_count);
        for _ in 0..cells_count {
            distances.push(Zero::zero());
        }

        // Wonder how this compares with standard Dijkstra shortest path tree algorithm
        // We don't have any weights on the edges/links to consider.
        //
        // push start_coordinate onto frontier set/vec/list
        // while frontier not empty
        //   new_frontier_set = []
        //   for cell in frontier
        //      for each linked cell
        //          ignore if already visited (distance number of that gridcoordinate != 0 && ! start)
        //          otherwise
        //          distance of cell = distance[cell] + 1
        //          add to new_frontier_set
        //   swap the frontier set to be that of the new_frontier_set
        let mut frontier = vec![*start_coordinate];
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
                    if distances[gc_index] != Zero::zero() {

                        distances[gc_index] = distance_to_cell + One::one();
                        new_frontier.push(*link);
                    }
                }
            }
            frontier = new_frontier;
        }

        // does it need to be a set? Can it be a Vec?
        // if a Vec can we implicitly convert the Vec index into a key?
        // the use of a set like structure stops the frontier expanding into itself sideways instead of outwards
        // A Vec would only work if GridCoordinate is put into a Vec Set of fixed capacity. A plain array might be better in that case.
        // For some workloads the Set could be a brute force vec search for cache efficiency.
        // The ruby code uses a list/vec as the set only exists to remove dupes...maybe the dupes would not matter and it's more efficient to
        // ignore them? The set would simply save on checks for whether a distance is already set.
        // The distances structure acts as a visited set aswell as a storer of the floodfill distances.
        // let mut set = fnv_hashset::<GridCoordinate>(cells_count/4);

        // also consider
        // arrayvec - vector with fixed capacity, can be on stack. ArrayVec and ArrayString.
        // smallvec - some items on the stack. E.g. let mut v = SmallVec::<[_; 16]>::new();
        // std::collections::VecDeque (growable ringbuffer)
        // std::collections::binary_heap (priority queue)
        // vec_map - integer index key into a Vec but more formal
        // linear_map - brute force vector map

        DijkstraDistances {
            grid: grid,
            start_coordinate: *start_coordinate,
            distances: distances,
        }
    }
}

fn fnv_hashset<T: Hash + Eq>(capacity: usize) -> HashSet<T, BuildHasherDefault<FnvHasher>> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashSet::<T, _>::with_capacity_and_hasher(capacity, fnv)
}

#[cfg(test)]
mod tests {

    // use super::*;

}
