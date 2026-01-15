// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Simple physical page allocator built around 64 KiB granules so the kernel can
// reserve deterministic spans before the MMU is enabled.

use alloc::vec::Vec;

pub const PAGE_SIZE: usize = 64 * 1024;
const PAGE_MASK: u64 = (PAGE_SIZE as u64) - 1;

#[inline(always)]
const fn align_down(value: u64) -> u64 {
    value & !PAGE_MASK
}

#[inline(always)]
const fn align_up(value: u64) -> u64 {
    if value & PAGE_MASK == 0 {
        value
    } else {
        (value & !PAGE_MASK) + (PAGE_SIZE as u64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RamRegion {
    pub start: u64,
    pub size: u64,
}

impl RamRegion {
    pub const fn new(start: u64, size: u64) -> Self {
        Self { start, size }
    }

    pub const fn end(&self) -> u64 {
        self.start + self.size
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PageSegment {
    start: u64,
    size: u64,
}

impl PageSegment {
    fn end(&self) -> u64 {
        self.start + self.size
    }
}

#[derive(Clone, Debug)]
pub struct ReservedRegion {
    pub start: u64,
    pub size: u64,
    pub kind: ReservationKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReservationKind {
    KernelImage,
    EmergencyPool,
    FirmwareArtifact,
    Custom(&'static str),
}

pub struct PageAllocator {
    free: Vec<PageSegment>,
    reserved: Vec<ReservedRegion>,
}

impl PageAllocator {
    pub fn from_regions(regions: &[RamRegion]) -> Self {
        let mut free = Vec::new();

        for region in regions {
            let start_aligned = align_up(region.start);
            let end_aligned = align_down(region.end());

            if end_aligned <= start_aligned {
                continue;
            }

            free.push(PageSegment {
                start: start_aligned,
                size: end_aligned - start_aligned,
            });
        }

        Self {
            free,
            reserved: Vec::new(),
        }
    }

    pub fn reserve_span(
        &mut self,
        start: u64,
        size: u64,
        kind: ReservationKind,
    ) -> Option<ReservedRegion> {
        if size == 0 {
            return None;
        }

        let aligned_start = align_down(start);
        let aligned_end = align_up(start + size);
        if aligned_end <= aligned_start {
            return None;
        }

        if !self.carve_free_segments(aligned_start, aligned_end) {
            return None;
        }

        let region = ReservedRegion {
            start: aligned_start,
            size: aligned_end - aligned_start,
            kind,
        };
        self.reserved.push(region.clone());
        Some(region)
    }

    pub fn allocate_contiguous(
        &mut self,
        page_count: usize,
        kind: ReservationKind,
    ) -> Option<ReservedRegion> {
        if page_count == 0 {
            return None;
        }

        let bytes = (page_count as u64) * (PAGE_SIZE as u64);

        for idx in 0..self.free.len() {
            let segment = &mut self.free[idx];
            if segment.size < bytes {
                continue;
            }

            let start = segment.start;
            segment.start += bytes;
            segment.size -= bytes;

            if segment.size == 0 {
                self.free.remove(idx);
            }

            let reserved = ReservedRegion {
                start,
                size: bytes,
                kind,
            };
            self.reserved.push(reserved.clone());
            return Some(reserved);
        }

        None
    }

    pub fn total_free_bytes(&self) -> u64 {
        self.free.iter().map(|segment| segment.size).sum()
    }

    pub fn reserved_regions(&self) -> &[ReservedRegion] {
        &self.reserved
    }

    fn carve_free_segments(&mut self, start: u64, end: u64) -> bool {
        let mut modified = false;
        let mut idx = 0;

        while idx < self.free.len() {
            let segment = self.free[idx];
            let seg_start = segment.start;
            let seg_end = segment.end();

            if end <= seg_start || start >= seg_end {
                idx += 1;
                continue;
            }

            modified = true;

            match (start <= seg_start, end >= seg_end) {
                (true, true) => {
                    self.free.remove(idx);
                    continue;
                }
                (true, false) => {
                    self.free[idx].start = end;
                    self.free[idx].size = seg_end - end;
                    idx += 1;
                }
                (false, true) => {
                    self.free[idx].size = start - seg_start;
                    idx += 1;
                }
                (false, false) => {
                    let tail = PageSegment {
                        start: end,
                        size: seg_end - end,
                    };
                    self.free[idx].size = start - seg_start;
                    self.free.insert(idx + 1, tail);
                    break;
                }
            }
        }

        modified
    }
}
