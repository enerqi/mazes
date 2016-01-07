use std::cell::{RefCell};
use std::rc::{Rc};

use cell::Cell;

// Rc is a pointer to the value T
// Heap allocated T with many readers
// . does DeRef automatically, though can manually deref with *
// clone is used to pass the ref-counted pointer
// but you can take a &T to the value inside, not needing to inc/dec the ref count
//     e.g. (&*myRcInstance)
// ofc you can also take a reference &T to the Rc pointer itself

// ! Weak<T> would allow passing a ref to a cell without worrying about cycles
// that we would have to clean up manually. Perhaps for the links between cells,
// but not for the grid/graph owning the cell and not if/when exposing the cell
// to end users as part of the public api.

// Rc RefCell T is best if we need a fully mutable graph
// OR want nodes to exist independently of the graph.
//
// RefCell allows the possiblity of mutation through the shared pointer (Rc)
//         using "interior mutability" and runtime borrow checks
pub type GridCell = Rc<RefCell<Cell>>;
