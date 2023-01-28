use crate::mmu::HostMemory;

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
    table: [Page; 0x10000],
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

    pub fn get_mut(&mut self, addr: u64) -> Option<&mut Page> {
        let p4_index = (addr >> 39) & 0x1FF;
        let p3_index = (addr >> 30) & 0x1FF;
        let p2_index = (addr >> 21) & 0x1FF;
        let p1_index = (addr >> 12) & 0x1FF;

        let p4 = self.root.table[p4_index as usize].as_mut()?;
        let p3 = p4.table[p3_index as usize].as_mut()?;
        let p2 = p3.table[p2_index as usize].as_mut()?;

        Some(&mut p2.table[p1_index as usize])
    }

    pub fn get_ref(&self, addr: u64) -> Option<&Page> {
        let p4_index = (addr >> 39) & 0x1FF;
        let p3_index = (addr >> 30) & 0x1FF;
        let p2_index = (addr >> 21) & 0x1FF;
        let p1_index = (addr >> 12) & 0x1FF;

        let p4 = self.root.table[p4_index as usize].as_ref()?;
        let p3 = p4.table[p3_index as usize].as_ref()?;
        let p2 = p3.table[p2_index as usize].as_ref()?;

        Some(&p2.table[p1_index as usize])
    }

    pub fn get_or_mmap<F>(&mut self, addr: u64, f: F) -> &mut Page
    where
        F: FnOnce() -> Page,
    {
        let p4_index = (addr >> 39) & 0x1FF;
        let p3_index = (addr >> 30) & 0x1FF;
        let p2_index = (addr >> 21) & 0x1FF;
        let p1_index = (addr >> 12) & 0x1FF;

        let p4 = self.root.table[p4_index as usize].get_or_insert_with(|| {
            Box::new(PageTableDepth2 {
                table: std::array::from_fn(|_| None),
            })
        });

        let p3 = p4.table[p3_index as usize].get_or_insert_with(|| {
            Box::new(PageTableDepth1 {
                table: std::array::from_fn(|_| None),
            })
        });

        let p2 = p3.table[p2_index as usize].get_or_insert_with(|| {
            Box::new(PageTableDepth0 {
                table: std::array::from_fn(|_| Page::Unmapped),
            })
        });

        p2.table[p1_index as usize] = f();
        &mut p2.table[p1_index as usize]
    }
}
