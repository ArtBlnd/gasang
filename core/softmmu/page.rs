use crate::softmmu::{HostMemory, PAGE_SIZE};

#[derive(Debug, Clone)]
pub enum Page {
    Unmapped,
    Memory {
        memory: HostMemory,

        readable: bool,
        writable: bool,
        executable: bool,
    },
}

#[derive(Debug)]
pub struct PageTableDepth0 {
    table: Box<[Page; 65536]>,
}

#[derive(Debug)]
pub struct PageTableDepth1 {
    table: [Option<PageTableDepth0>; 4096],
}

#[derive(Debug)]
pub struct PageTableDepth2 {
    table: [Option<Box<PageTableDepth1>>; 4096],
}

#[derive(Debug)]
pub struct PageTableDepth3 {
    table: [Option<Box<PageTableDepth2>>; 4096],
}

#[derive(Debug)]
pub struct PageTable {
    root: PageTableDepth3,
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            root: PageTableDepth3 {
                table: std::array::from_fn(|_| None),
            },
        }
    }

    fn as_offset(addr: u64) -> (usize, usize, usize, usize) {
        const D3_MASK: u64 = 0xFFF0_0000_0000_0000;
        const D2_MASK: u64 = 0x000F_FF00_0000_0000;
        const D1_MASK: u64 = 0x0000_00FF_F000_0000;
        const PG_MASK: u64 = 0x0000_0000_0FFF_F000;
        let d3_offset = ((addr & D3_MASK) >> 52) as usize;
        let d2_offset = ((addr & D2_MASK) >> 40) as usize;
        let d1_offset = ((addr & D1_MASK) >> 28) as usize;
        let pg_offset = ((addr & PG_MASK) >> 12) as usize;

        assert_eq!(D3_MASK | D2_MASK | D1_MASK | PG_MASK | (PAGE_SIZE-1), u64::MAX);
        assert_eq!(D3_MASK & D2_MASK & D1_MASK & PG_MASK & (PAGE_SIZE-1), 0);

        (d3_offset, d2_offset, d1_offset, pg_offset)
    }

    pub fn get_mem_offset(addr: u64) -> usize {
        (addr & 0x0000_0000_0000_0FFF) as usize
    }

    pub fn get_mut(&mut self, addr: u64) -> Option<&mut Page> {
        let (d3, d2, d1, pg) = Self::as_offset(addr);

        Some(
            &mut self.root.table[d3].as_mut()?.table[d2].as_mut()?.table[d1]
                .as_mut()?
                .table[pg],
        )
    }

    pub fn get_ref(&self, addr: u64) -> Option<&Page> {
        let (d3, d2, d1, pg) = Self::as_offset(addr);

        Some(
            &self.root.table[d3].as_ref()?.table[d2].as_ref()?.table[d1]
                .as_ref()?
                .table[pg],
        )
    }

    pub fn get_or_mmap<F>(&mut self, addr: u64, f: F) -> &mut Page
    where
        F: FnOnce() -> Page,
    {
        let (d3, d2, d1, pg) = Self::as_offset(addr);

        let pt = &mut self.root;
        let pt = pt.table[d3].get_or_insert_with(|| {
            Box::new(PageTableDepth2 {
                table: utility::make_array(|_| None),
            })
        });

        let pt = pt.table[d2].get_or_insert_with(|| {
            Box::new(PageTableDepth1 {
                table: utility::make_array(|_| None),
            })
        });

        let pt = pt.table[d1].get_or_insert_with(|| {
            let mut table = Vec::new();
            table.resize(65536, Page::Unmapped);

            PageTableDepth0 {
                table: table.into_boxed_slice().try_into().unwrap(),
            }
        });

        if let Page::Unmapped = pt.table[pg] {
            pt.table[pg] = f();
        }

        &mut pt.table[pg]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::softmmu::PAGE_SIZE;

    #[test]
    fn test_paging() {
        let mut table = PageTable::new();

        table.get_or_mmap(0xCAFE_BABE_DEAD_BEEE, || Page::Memory {
            memory: HostMemory::new(PAGE_SIZE as usize),
            readable: true,
            writable: true,
            executable: true,
        });

        let page = table.root;

        let page = page.table[3247].as_ref().unwrap();
        let page = page.table[3770].as_ref().unwrap();
        let page = page.table[3053].as_ref().unwrap();
        let _ = &page.table[60123];
    }
}
