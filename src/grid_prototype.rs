// Rust concerns
// Lifetime of the graph
//     - Can values within the graph outlive the graph?
//     - Can values even be referenced outside the graph?
//
// Graph mutability
//     - After initialisation, is the graph immutable?
//
// Graph ownership
//     - Unless it's a directed acyclic graph there is no unique owner
//
// Rc<RefCell<Node>>
// + most flexible allowing mutation and sharing of nodes outside the graph and anytime
// - slowest with runtime borrow checks
// - annoyingly unergonomic
//
// Arena allocation + UnsafeCell
// + Faster than Rc Refcell due to no runtime borrow checks
// + Compact and efficient as vectors/indices approach. Pointers (&) arguably more direct.
// + thread sharing tracked by borrow checker after init
// - All nodes exist as long as the Graph which are allocated from an allocator Arena.
// - Graph must be inited in an unsafe block. (still unergonomic in a different way).
// - adjacency list of nodes always accessed with unsafe block due to UnsafeCell (impl ugly point).
//
// Vectors and indices based approach
// + thread sharing tracked by borrow checker - unlike Rc Refcell
//           c.f. Arc<Mutex<RefCell>>
// + Compact, cache efficient, vector
// - type safety on indices helps. Client can accidentally mix up between graph instances.
// - deleting from the graph: freelist to reuse or placeholder just to tombstone
//
// Petgraph external Crate - uses vectors and indices
// Deletion is done by swapping the to delete index with the last one in the Vec.
// Max graph size is specified up front and seems to panic! when that is too big.
//
// Graph as Vec<LinkList<NodeIndex>> or Vec<Vec<NodeIndex>>
// - not as compact as vectors and indices as the adjacency lists for each node need
//   to be a linkedlist or a vec where we guess the initial size and the linkedlist
//   is allocator trigger happy.
// + allows parallel (multiple) edges between the same nodes
// + easier to delete a node without worrying about vector resize or freelists
//
// The maze algorithms seem to use some deleting - of the links, not the N,S,E,W refs
// I've already 'cheated' the borrow checker by using grid-coordinates for the links,
// on the assumption that cells (nodes) are not removed once created as a grid/graph.
// Do the refs need to be Rc RefCells?
//
// The square grids are composed of cells, each cell has a N, S, E, W relative to it.
// For polar grids we have rings of concentric circles
// Fox hex grids we have 6 directions N, S, NW, NE, SW, SE

//use std::cell::{RefCell};
//use std::rc::{Rc};
//use maze::cell;
//use maze::gridcell;

use rand;
use rand::distributions::{IndependentSample, Range};

// `use` statements are paths relative to the crate root module
// and cannot begin with ::
// When not using `use` paths are relative to the current module
// and `::` is relative to the crate root
use cell::{Cell}; // cannot use cell::* when there are circular imports
use gridcell::GridCell;

pub struct Grid {
    rows_count: usize,
    columns_count: usize,
    cells: Vec<GridCell>,
}

enum GridDirection {
    North,
    South,
    East,
    West,
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct GridCoordinate {
    pub x: isize,
    pub y: isize,
}

// This is perhaps pointless
// also what is the space usage of Rc<Refcell<Cell>>
// what benefit is there between Move vs Reference
pub struct Iter<'a> {
    cells: &'a Vec<GridCell>,
    current: usize,
    size: usize,
}
impl<'a> Iterator for Iter<'a> {
    type Item = &'a GridCell;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.size {
            let gridcell_ref = &self.cells[self.current];
            self.current += 1;
            Some(gridcell_ref)
        }
        else { None }
    }
}


impl Grid {
    pub fn new(scale: usize) -> Self {
        // The owner of data decides if it is mut or not
        // When we move it out by returning it, it can become mut or the default immutable
        let mut grid = Grid{
            rows_count: scale,
            columns_count: scale,
            cells: Vec::with_capacity(scale * scale)
        };

        for row_num in 0 .. scale {
            for column_num in 0 .. scale {
                let coord = GridCoordinate{x: column_num as isize, y: row_num as isize};
                grid.cells.push(Cell::new(coord).as_grid_cell());
            }
        }

        for cell in &grid.cells {
            // `cell` from the iterator is a '&T' reference.
            // we could have the api of neighbour_of take a `&GridCell`, though that maybe lacks uniformity/ease
            // vs `GridCell` which requires a clone of `cell`
            // let cell_north_of: Option<GridCell> = grid.neighbour_of(cell.clone(), GridDirection::North);
            // let cell_north_of: Option<GridCell> = grid.neighbour_of(cell, GridDirection::North);
            cell.borrow_mut().north = grid.neighbour_of(cell, GridDirection::North);
            cell.borrow_mut().south = grid.neighbour_of(cell, GridDirection::South);
            cell.borrow_mut().east = grid.neighbour_of(cell, GridDirection::East);
            cell.borrow_mut().west = grid.neighbour_of(cell, GridDirection::West);
        }

        grid
    }

    pub fn random_cell(&self) -> GridCell {

        let random_pos = Range::new(0, self.rows_count as isize);
        let mut rng = rand::thread_rng();
        let coord = GridCoordinate::new(random_pos.ind_sample(&mut rng),
                                        random_pos.ind_sample(&mut rng));

        self.cell_at(coord).unwrap() // Move the Gridcell pointer value
    }

    fn neighbour_of(&self, cell: &GridCell, dir: GridDirection) -> Option<GridCell> {
        let ref coord = cell.borrow().coordinate;
        let (x, y) = (coord.x, coord.y);
        match dir {
            GridDirection::North => self.cell_at(GridCoordinate::new(x, y - 1)),
            GridDirection::South => self.cell_at(GridCoordinate::new(x, y + 1)),
            GridDirection::East => self.cell_at(GridCoordinate::new(x + 1, y)),
            GridDirection::West => self.cell_at(GridCoordinate::new(x - 1, y)),
        }
    }

    fn cell_at(&self, coord: GridCoordinate) -> Option<GridCell> {
        if coord.x < 0 || coord.y < 0 {
            return None
        }

        let (x, y) = (coord.x as usize, coord.y as usize);

        if x >= self.columns_count ||
           y >= self.rows_count {
            return None
        }

        let index = (y * self.rows_count) + x;
        Some(self.cells[index].clone())
    }
}

impl GridCoordinate {
    pub fn new(x: isize, y: isize) -> Self {
        GridCoordinate {x: x, y: y}
    }
}



#[cfg(test)]
mod test {

}

// Cell Members__________________________________________
// The corners and edges of the grid will not have a neighbour in all 4 directions
// This maybe not the greatest way to do it in Rust, especially as we could easily
// calculate N/S/E/W neighbours from the row/column coordinate given information
// about the Grid
// It maybe more useful in a dynamic terrain/grid where the extents of the coordinates
// are not clearly known at compile time

// When defining a recursive struct or enum, any use of the type being defined
// from inside the definition must occur behind a pointer (like `Box` or `&`).
// This is because structs and enums must have a well-defined size, and without
// the pointer the size of the type would need to be unbounded.

// Even if we don't store what grid cells are N/S/E/W of us, we need to store what
// passages link to this one from the N/S/E/W as a Sequence
// Given that this is dynamically altered, 0-4 passages and refers to other grid cells
// we likely still need to deal with RefCells
// An alternative design would list the connected passages by their location instead of
// with a reference type
// A 2d location is clearly (x,y) 2 * usize (or could be i16 etc)
// What space is used for a reference type? Rc<RefCell<___>>?
// Rc allows a container (the RefCell) to be cloned and shared. Do we need to care about the RefCell's
// lifetime in this way? Yes..?
//
// pub struct Cell<T> {
//     value: UnsafeCell<T>,
// }
// So note, that RefCell itself is a value. The Ref instance accessible from a RefCell is what wraps a borrowed reference.
// Rc around the RefCell adds that pointer indirection to the value, so we don't have multiple copies of the value
// As far as memory goes, Rc<T> is a single allocation, though it will allocate two extra words as compared to a
// regular Box<T> (for “strong” and “weak” refcounts).
// Rc<T> has the computational cost of incrementing/decrementing the refcount whenever it is cloned or goes out of scope
// respectively. Note that a clone will not do a deep copy, rather it will simply increment the inner reference count and return a copy of the Rc<T>
//
// &RefCell? Not sure that makes any sense, though it is possible if Every Cell in the Grid is a RefCell<Cell>
// Then, we can give out shared references '&RefCell'
// The references to the RefCell<Cell> would be tangled, but the borrow checker won't care if we drop the whole grid of
// Cells and during the destruction period some of the &RefCell<Cell> are uninitialized????? Actually it probably will complain
// as the & part of '&RefCell' will still do lifetime related checks, which is different to the shared mutability checks that
// we have avoided at compile time by using RefCell<Cell>
//
// Therefore, Rc<RefCell<Cell>> seems to be what we need.
// Does the Grid use Rc as well?
// Does Rc have problems with cycles in this design????
//
// Cells would not know about the Grid
// Grid owns Cells? Makes sense if we want the grid to be able to find a predictable Cell given a coordinate.
// If Cells ref other Cells with an Rc<RefCell>,
//
// Well a plain RefCell won't work because of the need to know the size of the type at compile time for a recursive type
// Cell Members^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

// Box = allocate on the heap and drop it when the Box goes out of scope.
// Rc and Arc = shared smart pointer type - inc ref count when cloned. This lets us have multiple “owning” pointers to the same data.
//              The main guarantee provided here is that the data will not be destroyed until all references to it are out of scope.
//              When you wish to dynamically allocate and share some data (read-only) between various portions of your program,
//              where it is not certain which portion will finish using the pointer last
// (1)    (2+) threads
// Weak = non-owning, no lifetime restriction. Useful with cyclic data.
// RefCell and Cell = allow mutability of shared references with borrow checking done at runtime
// (clone)     (copy) Often held inside Rc otherwise the shared reference would have no way to be mutated
//                    These types are generally found in struct fields, but they may be found elsewhere too.
//                    RefCell does not allocate, but it contains an additional “borrow state” indicator (one word in size) along with the data.
//                    At runtime each borrow causes a modification/check of the refcount.
// Mutex = shared mutability in multi-threading
// (2+) thread
// &mut T = statically unique reference
// &T = shared reference, statically checked for borrows
//
// Cell does NOT have a fixed size, so we need to indirect via a pointer to its neighbours - a '&' or something else
// If the neighbour links are all `&Cell`, then they cannot be changed whilst the Links exist, at least through these references
// Initially the Links would all be `None`
// What happens when destroy all the Cells on the stack, would it not invalidate an existing reference?
// This is more complicated than a singly linked list as we can have multiple Links to One Cell and we have multiple out links from one cell

//type NeighbourGridCell = Option<Rc<RefCell<Cell>>>;

// we need all the lifetime references here:
// '&': need to ensure the thing we reference will last long enough
// 'Cell': ensure the Cell value behind the RefCell lasts
//type NeighbourGridCell<'a> = Option<&'a RefCell<Cell<'a>>>;

// Ok new version...
// The RefCell is a value, slightly larger than the value it wraps
// If we create a '& RefCell' then we have the issue of where to store the
// RefCell data. If the Cell in the grid is a non-refcell but the neighbours refs
// are not, we could screw up and have 2 mutable refs active
// If we do want to do the RefCell<&Cell> approach then the RefCell could be Rc<RefCell<&Cell>>
// but even then...creating the actual RefCell on the stack has a lifetime problem
//type NeighbourGridCell = Option<&'a RefCell<Cell<'a>>>

// Ok version 3
// Option<&'a RefCell<Cell<'a>>> has a problem with the borrowed '&' only being valid for the block that we
// initialise the Grid in. Certainly if the vector reallocates, a stored & to a refcell shouldn't work
// so back to the full Rc RefCell
// This should also mean that we can remove the lifetime 'a annotations
//type NeighbourGridCell = Option<Rc<RefCell<Cell>>>;
