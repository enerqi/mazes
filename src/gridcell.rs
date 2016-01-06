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

// RefCell allows the possiblity of mutation through the shared pointer (Rc)
pub type GridCell = Rc<RefCell<Cell>>;
