#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

/// Early memory allocator.
///
/// It is a very simple allocator used before a formal byte allocator and page
/// allocator can work. It manages a memory range from both ends:
///
/// - Bytes are allocated **forward** from the start of the range.
/// - Pages are allocated **backward** from the end of the range.
///
/// The memory layout is as follows:
///
/// ```text
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
/// ```
///
/// For the bytes area, `count` records the number of alive allocations. When
/// it goes down to ZERO, the whole bytes-used area is freed (i.e., `b_pos` is
/// reset to `start`), so subsequent allocations can reuse it.
///
/// For the pages area, memory will never be freed.
pub struct EarlyAllocator<const SIZE: usize> {
    /// Start address of the managed memory range.
    start: usize,
    /// End address (exclusive) of the managed memory range.
    end: usize,
    /// Current boundary of the forward-growing bytes area.
    b_pos: usize,
    /// Current boundary of the backward-growing pages area.
    p_pos: usize,
    /// Number of alive byte allocations.
    count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    /// Creates a new empty `EarlyAllocator`.
    ///
    /// It must be initialized by [`BaseAllocator::init`] before use.
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }

    /// Returns `true` if this allocator has been initialized.
    fn is_inited(&self) -> bool {
        self.end > self.start
    }
}

impl<const SIZE: usize> Default for EarlyAllocator<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

/// Aligns `pos` upwards to `align`. `align` must be a power of two.
#[inline]
const fn align_up(pos: usize, align: usize) -> Option<usize> {
    match pos.checked_add(align - 1) {
        Some(v) => Some(v & !(align - 1)),
        None => None,
    }
}

/// Aligns `pos` downwards to `align`. `align` must be a power of two.
#[inline]
const fn align_down(pos: usize, align: usize) -> usize {
    pos & !(align - 1)
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(size > 0, "the size of the early allocator must be positive");
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        // The early allocator cannot manage non-contiguous memory regions.
        Err(AllocError::NoMemory)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        if !self.is_inited() || layout.align() == 0 {
            return Err(AllocError::InvalidParam);
        }

        // All the previously allocated bytes have been freed, so the whole
        // bytes-used area can be reused.
        if self.count == 0 {
            self.b_pos = self.start;
        }

        let alloc_start = align_up(self.b_pos, layout.align()).ok_or(AllocError::NoMemory)?;
        let alloc_end = alloc_start
            .checked_add(layout.size())
            .ok_or(AllocError::NoMemory)?;
        if alloc_end > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos = alloc_end;
        self.count += 1;
        NonNull::new(alloc_start as *mut u8).ok_or(AllocError::NoMemory)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        assert!(self.count > 0, "no allocation to deallocate");
        self.count -= 1;
        if self.count == 0 {
            // Free the whole bytes-used area.
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    fn used_bytes(&self) -> usize {
        if self.count == 0 {
            0
        } else {
            self.b_pos.saturating_sub(self.start)
        }
    }

    fn available_bytes(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos)
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if !self.is_inited() || align_pow2 == 0 || !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        if num_pages == 0 {
            return Err(AllocError::InvalidParam);
        }

        let size = num_pages.checked_mul(SIZE).ok_or(AllocError::NoMemory)?;
        let alloc_end = self.p_pos.checked_sub(size).ok_or(AllocError::NoMemory)?;
        let alloc_start = align_down(alloc_end, align_pow2);
        if alloc_start < self.b_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos = alloc_start;
        Ok(alloc_start)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // The pages-used area is never freed.
    }

    fn total_pages(&self) -> usize {
        self.end.saturating_sub(self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        self.end.saturating_sub(self.p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos) / SIZE
    }
}
