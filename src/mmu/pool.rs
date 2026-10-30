use crate::mmu::{MmuError, NUM_L2_ENTRIES, NUM_POOL_ELEMENTS, PageTable};
use core::cell::{RefCell, UnsafeCell};
use critical_section::Mutex;

static PAGE_TABLE_ARRAY: PageTableArray<NUM_POOL_ELEMENTS> = PageTableArray::new();

static SYS_PAGE_TABLE_POOL: Mutex<RefCell<Option<PageTablePool<NUM_POOL_ELEMENTS>>>> =
    Mutex::new(RefCell::new(None));

#[repr(C, align(1024))] // 1KiB alignment
struct PageTableArray<const N: usize> {
    array: UnsafeCell<[[u32; NUM_L2_ENTRIES]; N]>,
}

impl<const N: usize> PageTableArray<N> {
    const fn new() -> Self {
        PageTableArray {
            array: UnsafeCell::new([[0u32; NUM_L2_ENTRIES]; N]),
        }
    }

    const fn get_ptr(&self, idx: usize) -> *mut [u32; NUM_L2_ENTRIES] {
        unsafe { self.array.get().cast::<[u32; NUM_L2_ENTRIES]>().add(idx) }
    }
}

unsafe impl<const N: usize> Sync for PageTableArray<N> {}

struct PageTablePool<'a, const N: usize> {
    page_table_array: &'a PageTableArray<N>,
    alloc_idx: usize,
}

impl<'a, const N: usize> PageTablePool<'a, N> {
    const fn new(array: &'a PageTableArray<N>) -> PageTablePool<'a, N> {
        PageTablePool {
            page_table_array: array,
            alloc_idx: 0,
        }
    }

    fn new_page_table(&mut self) -> Result<*mut [u32; NUM_L2_ENTRIES], MmuError> {
        if self.alloc_idx < N {
            let idx = self.alloc_idx;
            self.alloc_idx += 1;
            Ok(self.page_table_array.get_ptr(idx))
        } else {
            Err(MmuError::NoFreePages)
        }
    }
}

unsafe impl<const N: usize> Sync for PageTablePool<'_, N> {}

pub(super) fn sys_alloc_page_table() -> Result<PageTable<'static>, MmuError> {
    fn alloc_page_table() -> Result<*mut [u32; NUM_L2_ENTRIES], MmuError> {
        critical_section::with(|cs| {
            if let Some(sys_pool) = SYS_PAGE_TABLE_POOL.borrow(cs).borrow_mut().as_mut() {
                sys_pool.new_page_table()
            } else {
                Err(MmuError::Others)
            }
        })
    }

    alloc_page_table().map(|addr| PageTable {
        descs: unsafe { &mut *addr },
    })
}

pub(super) fn init() {
    critical_section::with(|cs| {
        SYS_PAGE_TABLE_POOL
            .borrow(cs)
            .replace(Some(PageTablePool::new(&PAGE_TABLE_ARRAY)));
    });
}
