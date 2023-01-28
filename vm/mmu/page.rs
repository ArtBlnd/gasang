use crate::mmu::{HostMemory, PAGE_SIZE};

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
    table: [Page; 4096],
}

#[derive(Debug)]
pub struct PageTableDepth1 {
    table: [Option<Box<PageTableDepth0>>; 4096],
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

    pub fn get_mem_offset(addr: u64) -> usize {
        (addr & 0x0000_0000_0000_FFFF) as usize
    }

    pub fn get_mut(&mut self, addr: u64) -> Option<&mut Page> {
        let d3_offset = ((addr & 0xFFF0_0000_0000_0000) >> 52) as usize;
        let d2_offset = ((addr & 0x000F_FF00_0000_0000) >> 40) as usize;
        let d1_offset = ((addr & 0x0000_00FF_F000_0000) >> 28) as usize;
        let pg_offset = ((addr & 0x0000_0000_0FFF_0000) >> 16) as usize;

        Some(
            &mut self.root.table[d3_offset].as_mut()?.table[d2_offset]
                .as_mut()?
                .table[d1_offset]
                .as_mut()?
                .table[pg_offset],
        )
    }

    pub fn get_ref(&self, addr: u64) -> Option<&Page> {
        let d3_offset = ((addr & 0xFFF0_0000_0000_0000) >> 52) as usize;
        let d2_offset = ((addr & 0x000F_FF00_0000_0000) >> 40) as usize;
        let d1_offset = ((addr & 0x0000_00FF_F000_0000) >> 28) as usize;
        let pg_offset = ((addr & 0x0000_0000_0FFF_0000) >> 16) as usize;

        Some(
            &self.root.table[d3_offset].as_ref()?.table[d2_offset]
                .as_ref()?
                .table[d1_offset]
                .as_ref()?
                .table[pg_offset],
        )
    }

    pub fn get_or_mmap<F>(&mut self, addr: u64, f: F) -> &mut Page
    where
        F: FnOnce() -> Page,
    {
        let d3_offset = ((addr & 0xFFF0_0000_0000_0000) >> 52) as usize;
        let d2_offset = ((addr & 0x000F_FF00_0000_0000) >> 40) as usize;
        let d1_offset = ((addr & 0x0000_00FF_F000_0000) >> 28) as usize;
        let pg_offset = ((addr & 0x0000_0000_0FFF_0000) >> 16) as usize;

        let pt = &mut self.root;
        let pt = pt.table[d3_offset].get_or_insert_with(|| {
            Box::new(PageTableDepth2 {
                table: std::array::from_fn(|_| None),
            })
        });

        let pt = pt.table[d2_offset].get_or_insert_with(|| {
            Box::new(PageTableDepth1 {
                table: std::array::from_fn(|_| None),
            })
        });

        let pt = pt.table[d1_offset].get_or_insert_with(|| {
            Box::new(PageTableDepth0 {
                table: std::array::from_fn(|_| Page::Unmapped),
            })
        });

        if let Page::Unmapped = pt.table[pg_offset] {
            pt.table[pg_offset] = f();
        }

        &mut pt.table[pg_offset]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
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
        let _ = &page.table[3757];
    }
}
