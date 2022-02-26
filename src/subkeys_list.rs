use binread::{derive_binread, BinRead};

/// On-Disk Structure of a Subkeys List header.
/// This is common for all subkey types (Fast Leaf, Hash Leaf, Index Leaf, Index Root).
#[derive_binread]
#[br(little)]
pub(crate) enum SubKeysList {

    #[br(magic = b"li")] IndexLeaf{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<IndexLeafItem>
    },

    #[br(magic = b"lf")] FastLeaf{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<FastLeafItem>
    },

    #[br(magic = b"lh")] HashLeaf{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<HashLeafItem>
    },

    #[br(magic = b"ri")] IndexRoot{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<IndexRootListElement>
    },
}

impl SubKeysList {
    pub fn offsets<'a>(&'a self) -> Box<dyn Iterator<Item=u32> + 'a> {
        match self {
            SubKeysList::IndexLeaf { items} => Box::new(items.iter().map(|i| i.key_node_offset)),
            SubKeysList::FastLeaf { items } => Box::new(items.iter().map(|i| i.key_node_offset)),
            SubKeysList::HashLeaf { items } => Box::new(items.iter().map(|i| i.key_node_offset)),
            SubKeysList::IndexRoot { items } => Box::new(items.iter().map(|i| i.subkeys_list_offset)),
        }
    }

    pub fn into_offsets<'a>(self) -> Box<dyn Iterator<Item=u32>> {
        match self {
            SubKeysList::IndexLeaf { items} => Box::new(items.into_iter().map(|i| i.key_node_offset)),
            SubKeysList::FastLeaf { items } => Box::new(items.into_iter().map(|i| i.key_node_offset)),
            SubKeysList::HashLeaf { items } => Box::new(items.into_iter().map(|i| i.key_node_offset)),
            SubKeysList::IndexRoot { items } => Box::new(items.into_iter().map(|i| i.subkeys_list_offset)),
        }
    }

    pub fn is_index_root(&self) -> bool {
        matches!(self, SubKeysList::IndexRoot { items })
    }
}

#[derive(BinRead)]
pub(crate) struct HashLeafItem {
    key_node_offset: u32,
    name_hash: [u8; 4],
}

#[derive(BinRead)]
pub(crate) struct FastLeafItem {
    key_node_offset: u32,
    name_hint: [u8; 4],
}

#[derive(BinRead)]
pub(crate) struct IndexRootListElement {
    subkeys_list_offset: u32
}

#[derive(BinRead)]
pub(crate) struct IndexLeafItem {
    key_node_offset: u32,
}