use std::cell::{RefCell};
use std::rc::{Rc};
use std::collections::HashSet;

use gridcell::{GridCell};
use grid::{GridCoordinate};

// splitting Grid and Cell into different files and hence different modules
// is annoying in that the members are now all public, which is fine within
// the crate maybe, but can we not do that for modules in other crates?
// Similar to java package visibility vs public

pub struct Cell {
    pub coordinate: GridCoordinate,
    pub north: Option<GridCell>,
    pub south: Option<GridCell>,
    pub east: Option<GridCell>,
    pub west: Option<GridCell>,
    links: HashSet<GridCoordinate>, // GridCells are RefCells etc, which probably won't have a stable hash, so tracking links via coordinates
}

pub enum LinkType {
    UniDirectional,
    BiDirectional,
}

impl Cell {

    pub fn new(coord: GridCoordinate) -> Self {
        Cell {coordinate: coord,
              north: None, south: None, east: None, west: None,
              links: HashSet::new()}
    }

    // move into a GridCell
    pub fn as_grid_cell(self) -> GridCell {
        Rc::new(RefCell::new(self))
    }

    pub fn link(&mut self, link_cell: &mut Cell, link_type: LinkType) {
    //pub fn link(&mut self, cell: &mut GridCell, link_type: LinkType) {
        //let coord: GridCoordinate = cell.borrow().coordinate; // We don't need clone, we have derived Copy
        let coord = link_cell.coordinate;
        self.links.insert(coord);

        if let LinkType::BiDirectional = link_type {
             link_cell.link(self, LinkType::UniDirectional);
        }
    }

    pub fn unlink(&mut self, unlink_cell: &mut Cell, link_type: LinkType) {
        let coord = unlink_cell.coordinate;
        self.links.remove(&coord);

        if let LinkType::BiDirectional = link_type {
            unlink_cell.unlink(self, LinkType::UniDirectional);
        }
    }

    pub fn links(&self) -> Vec<GridCoordinate> {
        // we could return an immutable reference to the HashSet or make a copy
        // error: the trait `core::iter::FromIterator<&grid::GridCoordinate>` is not implemented for the type `collections::vec::Vec<grid::GridCoordinate>`
        // means...
        //  "the type you are iterating over doesn't match the expected type of the collection you are creating"? ???
        // Vec<&GridCoordinate> vs Vec<GridCoordinate>
        //let links: Vec<&GridCoordinate> = self.links.iter().collect();
        //let links: Vec<GridCoordinate> = self.links.into_iter().collect();
        // Just taking a copy might work by using into_iter, but then we would need &mut self
        let links: Vec<GridCoordinate> = self.links.iter().map(|&coord| coord).collect();
        links
    }

    pub fn neighbours(&self) -> Vec<GridCell> {
        let maybe_cells: Vec<Option<&GridCell>> = vec![self.north.as_ref(), self.south.as_ref(),
                                                       self.east.as_ref(), self.west.as_ref()];
        // as_ref converts Option<GridCell> to Option<&GridCell> which we can move without messing up the original
        // &Option<GridCell> just doesn't seem to play nice
        //let filtered = maybe_cells.iter().filter(|&opt| opt.is_some());
        //let cells: Vec<GridCell> = filtered.map(|&opt| opt.unwrap().clone()).collect();
        let cells = maybe_cells.iter().filter(|&opt| opt.is_some()).map(|&opt| opt.unwrap().clone()).collect();
        //let cells = maybe_cells.iter().filter_map(|opt| opt.unwrap().clone()).collect();
        cells
    }
}
