use std::{rc::Rc, cell::RefCell};
use nt_hive2::{Offset, KeyNode};

pub (crate) struct RegTreeEntry {
    offset: Offset,
    nk: KeyNode,
    children: Vec<Rc<RefCell<Self>>>,
}

impl RegTreeEntry {
    pub fn new(offset: Offset, nk: KeyNode) -> Self {
        Self { offset, nk, children: Vec::new() }
    }

    pub fn add_child(&mut self, child: Rc<RefCell<Self>>) {
        self.children.push(child);
    }
}
