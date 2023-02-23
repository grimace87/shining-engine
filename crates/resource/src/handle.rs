
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Handle {
    table_index: u32,
    unique_id: u32
}

impl Handle {

    // #[inline]
    // pub fn with_unique_id(index: u32, unique_id: u32) -> Handle {
    //     Handle {
    //         table_index: index,
    //         unique_id
    //     }
    // }

    #[inline]
    pub fn for_resource(index: u32) -> Handle {
        Handle {
            table_index: index,
            unique_id: 0
        }
    }

    /// Construct a new handle where the unique ID is not important but we instead want to
    /// store multiple handles with what looks like the same (virtual) table index. This is done by
    /// passing a variation number separately.
    /// The variation number must use only two bits.
    #[inline]
    pub fn for_resource_variation(index: u32, variation: u32) -> Option<Handle> {
        if variation >= 0x4 || index >= 0x40000000 {
            return None;
        }
        let table_index = (index << 4) | variation;
        Some(Handle {
            table_index,
            unique_id: 0
        })
    }

    #[inline]
    pub fn table_index(&self) -> u32 {
        self.table_index
    }

    #[inline]
    pub fn unique_id(&self) -> u32 {
        self.unique_id
    }
}
