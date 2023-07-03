use std::{rc::Rc, cell::RefCell, collections::{HashMap, hash_map}};

use binread::{BinReaderExt};
use nt_hive2::{Offset, Hive, KeyNode, CleanHive};

use crate::regtreeentry::*;

pub (crate) struct RegTreeBuilder {
    /// contains all RegTreeEntrys with no parent
    orphan_entries: HashMap<Offset, Rc<RefCell<RegTreeEntry>>>,

    /// contains all (really all) RegTreeEntrys
    entries: HashMap<Offset, Rc<RefCell<RegTreeEntry>>>,

    /// contains the offsets of all non-added entries which are parents of
    /// already added entries, together with the entries that miss their parents
    missing_parents: HashMap<Offset, Vec<Rc<RefCell<RegTreeEntry>>>>
}

impl<B> From<Hive<B, CleanHive>> for RegTreeBuilder where B: BinReaderExt {
    fn from(hive: Hive<B, CleanHive>) -> Self {
        Self::from_hive(hive, |_| ())
    }
}

pub (crate) struct RootNodes<'a> {
    values: hash_map::Values<'a, Offset, Rc<RefCell<RegTreeEntry>>>,
}

impl<'a> Iterator for RootNodes<'a> {
    type Item = &'a Rc<RefCell<RegTreeEntry>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next()
    }
}


impl RegTreeBuilder {

    pub fn from_hive<B, C>(hive: Hive<B, CleanHive>, progress_callback: C) -> Self where B: BinReaderExt, C: Fn(u64) {
        let iterator = hive.into_cell_iterator(progress_callback);
        let mut me = Self {
            orphan_entries: HashMap::new(),
            entries: HashMap::new(),
            missing_parents: HashMap::new(),
        };

        let mut last_offset = Offset(0);
        for cell in iterator {
            let my_offset = *cell.offset();
            let is_deleted = cell.header().is_deleted();
            assert_ne!(last_offset, my_offset);
            log::trace!("found new cell at offset 0x{:x}", my_offset.0);

            if let Ok(nk) = TryInto::<KeyNode>::try_into(cell) {
                me.insert_nk(my_offset, nk, is_deleted)
            };

            last_offset = my_offset;
        }
        me
    }

    pub fn root_nodes(&self) -> RootNodes {
        RootNodes {
            values: self.orphan_entries.values()
        }
    }
    
    /// add a [`KeyNode`], found at [`Offset`] `nk_offset`, to the entries of `self`.
    /// This method tries to determine the complete path of the [`KeyNode`].
    fn insert_nk(&mut self, nk_offset: Offset, nk: KeyNode, is_deleted: bool) {
        assert!(! self.orphan_entries.contains_key(&nk_offset));
        assert!(! self.entries.contains_key(&nk_offset));

        let parent_offset = nk.parent;
        let entry = Rc::new(RefCell::new(RegTreeEntry::new(nk_offset, nk, is_deleted)));
        self.entries.insert(nk_offset, Rc::clone(&entry));
        
        // check if the parent of the current node has already been added.
        // If yes, than put the current node below of it. If not, add the
        // current node at the root level (which can contain more than one nodes)
        let key_has_parents =
        match self.entries.get(&parent_offset) {
            Some(parent_entry) => {
                assert!(! self.missing_parents.contains_key(&parent_offset));
                parent_entry.borrow_mut().add_child(Rc::clone(&entry));
                true
            },
            None => {
                self.orphan_entries.insert(nk_offset, Rc::clone(&entry));
                self.add_child_that_misses_parent(Rc::clone(&entry), parent_offset);
                false
            }
        };

        // check if the current node has children which have already been
        // added. If yes, those children should've been at the root level   
        // until now and must be reordered
        if let Some(children) = self.missing_parents.remove(&nk_offset) {
            for child in children.into_iter() {
                entry.borrow_mut().add_child(child);
            }
        }

        debug_assert!(self.entries.contains_key(&nk_offset));
        if key_has_parents {
            debug_assert!(! self.orphan_entries.contains_key(&nk_offset));
            debug_assert!(self.entries.get(&parent_offset).unwrap().borrow().has_child(nk_offset));
        } else {
            debug_assert!(self.orphan_entries.contains_key(&nk_offset));
        }

        self.entries.insert(nk_offset, entry);
    }

    fn add_child_that_misses_parent(&mut self, child: Rc<RefCell<RegTreeEntry>>, parent_offset: Offset) {
        match self.missing_parents.get_mut(&parent_offset) {
            Some(parent) => {
                parent.push(child);
            },
            None => {
                self.missing_parents.insert(parent_offset, vec![child]);
            }
        }
    }
}