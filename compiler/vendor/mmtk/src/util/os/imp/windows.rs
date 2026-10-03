//! Windows virtual memory for MMTk. The collector is still MMTk; this module is the
//! missing `OS` implementation for `x86_64-pc-windows-msvc`.

use crate::util::address::Address;
use crate::util::constants::BYTES_IN_PAGE;
use crate::util::os::*;
use std::io::Result;

const MEM_COMMIT: u32 = 0x1000;
const MEM_RESERVE: u32 = 0x2000;
const MEM_DECOMMIT: u32 = 0x4000;
const MEM_RELEASE: u32 = 0x8000;
const PAGE_NOACCESS: u32 = 0x01;
const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;

extern "system" {
    fn VirtualAlloc(addr: *mut libc::c_void, size: usize, typ: u32, prot: u32) -> *mut libc::c_void;
    fn VirtualFree(addr: *mut libc::c_void, size: usize, typ: u32) -> i32;
    fn GetCurrentProcessId() -> u32;
    fn GetCurrentThreadId() -> u32;
    fn GetCurrentThread() -> *mut libc::c_void;
    fn SetThreadAffinityMask(thread: *mut libc::c_void, mask: usize) -> usize;
}

pub struct Windows;

fn page_prot(prot: MmapProtection) -> u32 {
    match prot {
        MmapProtection::ReadWrite => PAGE_READWRITE,
        MmapProtection::ReadWriteExec => PAGE_EXECUTE_READWRITE,
        MmapProtection::NoAccess => PAGE_NOACCESS,
    }
}

fn alloc_flags(strategy: MmapStrategy) -> u32 {
    if strategy.reserve {
        MEM_RESERVE
    } else {
        MEM_RESERVE | MEM_COMMIT
    }
}

fn map_at(
    start: Address,
    size: usize,
    strategy: MmapStrategy,
    annotation: &MmapAnnotation<'_>,
) -> MmapResult<Address> {
    let prot = if strategy.reserve {
        PAGE_NOACCESS
    } else {
        page_prot(strategy.prot)
    };
    let ptr = unsafe { VirtualAlloc(start.to_mut_ptr(), size, alloc_flags(strategy), prot) };
    if ptr.is_null() {
        return Err(MmapError::new(
            start,
            size,
            annotation,
            std::io::Error::last_os_error(),
        ));
    }
    Ok(Address::from_mut_ptr(ptr))
}

fn map_aligned(
    preferred: Address,
    size: usize,
    align: usize,
    strategy: MmapStrategy,
    annotation: &MmapAnnotation<'_>,
) -> MmapResult<Address> {
    let align = align.max(BYTES_IN_PAGE);
    let alloc_size = size.saturating_add(align);
    let hint = if preferred.is_zero() {
        std::ptr::null_mut()
    } else {
        preferred.to_mut_ptr()
    };
    let prot = if strategy.reserve {
        PAGE_NOACCESS
    } else {
        page_prot(strategy.prot)
    };
    let ptr = unsafe { VirtualAlloc(hint, alloc_size, alloc_flags(strategy), prot) };
    if ptr.is_null() {
        return Err(MmapError::new(
            preferred,
            alloc_size,
            annotation,
            std::io::Error::last_os_error(),
        ));
    }
    let start = Address::from_mut_ptr(ptr);
    let aligned = start.align_up(align);
    let lead = aligned - start;
    if lead > 0 {
        unsafe {
            VirtualFree(start.to_mut_ptr(), lead, MEM_DECOMMIT);
        }
    }
    let tail = alloc_size - lead - size;
    if tail > 0 {
        let tail_start = aligned + size;
        unsafe {
            VirtualFree(tail_start.to_mut_ptr(), tail, MEM_DECOMMIT);
        }
    }
    Ok(aligned)
}

impl OSMemory for Windows {
    fn dzmmap(
        start: Address,
        size: usize,
        strategy: MmapStrategy,
        annotation: &MmapAnnotation<'_>,
    ) -> MmapResult<Address> {
        map_at(start, size, strategy, annotation)
    }

    fn dzmmap_anywhere(
        size: usize,
        align: usize,
        strategy: MmapStrategy,
        annotation: &MmapAnnotation<'_>,
    ) -> MmapResult<Address> {
        map_aligned(Address::ZERO, size, align, strategy, annotation)
    }

    fn dzmmap_preferred(
        start: Address,
        size: usize,
        align: usize,
        strategy: MmapStrategy,
        annotation: &MmapAnnotation<'_>,
    ) -> MmapResult<Address> {
        map_aligned(start, size, align, strategy, annotation)
    }

    fn munmap(start: Address, size: usize) -> Result<()> {
        let released = unsafe { VirtualFree(start.to_mut_ptr(), 0, MEM_RELEASE) };
        if released != 0 {
            return Ok(());
        }
        let decommitted = unsafe { VirtualFree(start.to_mut_ptr(), size, MEM_DECOMMIT) };
        if decommitted != 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    fn set_memory_access(start: Address, size: usize, prot: MmapProtection) -> Result<()> {
        if matches!(prot, MmapProtection::NoAccess) {
            let ok = unsafe { VirtualFree(start.to_mut_ptr(), size, MEM_DECOMMIT) };
            return if ok != 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            };
        }
        let ptr = unsafe { VirtualAlloc(start.to_mut_ptr(), size, MEM_COMMIT, page_prot(prot)) };
        if ptr.is_null() {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn is_mmap_oom(os_errno: i32) -> bool {
        os_errno == 8
    }

    fn panic_if_unmapped(_start: Address, _size: usize) {}
}

impl OSProcess for Windows {
    type ProcessIDType = u32;
    type ThreadIDType = u32;

    fn get_process_memory_maps() -> Result<String> {
        Ok(String::new())
    }

    fn get_process_id() -> Result<Self::ProcessIDType> {
        Ok(unsafe { GetCurrentProcessId() })
    }

    fn get_thread_id() -> Result<Self::ThreadIDType> {
        Ok(unsafe { GetCurrentThreadId() })
    }

    fn get_total_num_cpus() -> CoreNum {
        std::thread::available_parallelism()
            .map(|n| n.get() as CoreNum)
            .unwrap_or(1)
    }

    fn bind_current_thread_to_core(core_id: CoreId) {
        if core_id < 64 {
            unsafe {
                SetThreadAffinityMask(GetCurrentThread(), 1usize << core_id);
            }
        }
    }

    fn bind_current_thread_to_cpuset(core_ids: &[CoreId]) {
        let mut mask = 0usize;
        for core in core_ids {
            if *core < 64 {
                mask |= 1usize << core;
            }
        }
        if mask != 0 {
            unsafe {
                SetThreadAffinityMask(GetCurrentThread(), mask);
            }
        }
    }
}
