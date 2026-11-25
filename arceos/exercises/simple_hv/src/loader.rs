use std::io::{self, Read};
use std::fs::File;
use alloc::vec;
use axhal::paging::MappingFlags;
use axhal::mem::{PAGE_SIZE_4K, phys_to_virt};
use axmm::AddrSpace;
use crate::VM_ENTRY;

/// Loads a VM image into the guest address space.
///
/// The guest image is loaded into a single page at [`VM_ENTRY`]. The whole page
/// is zeroed first, then the image content is copied in, so that the tail of
/// the page is deterministic instead of containing stale stack data.
pub fn load_vm_image(fname: &str, uspace: &mut AddrSpace) -> io::Result<()> {
    let mut buf = vec![0u8; PAGE_SIZE_4K];
    let n = load_file(fname, &mut buf)?;
    if n == 0 {
        return Err(io::Error::InvalidInput);
    }
    ax_println!("load {} bytes into guest memory", n);

    uspace
        .map_alloc(
            VM_ENTRY.into(),
            PAGE_SIZE_4K,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE | MappingFlags::USER,
            true,
        )
        .unwrap();

    let (paddr, _, _) = uspace
        .page_table()
        .query(VM_ENTRY.into())
        .unwrap_or_else(|_| panic!("Mapping failed for segment: {:#x}", VM_ENTRY));

    ax_println!("paddr: {:#x}", paddr);

    unsafe {
        core::ptr::copy_nonoverlapping(
            buf.as_ptr(),
            phys_to_virt(paddr).as_mut_ptr(),
            n.min(PAGE_SIZE_4K),
        );
    }

    Ok(())
}

fn load_file(fname: &str, buf: &mut [u8]) -> io::Result<usize> {
    ax_println!("app: {}", fname);
    let mut file = File::open(fname)?;
    let mut read_len = 0;
    while read_len < buf.len() {
        let n = file.read(&mut buf[read_len..])?;
        if n == 0 {
            break;
        }
        read_len += n;
    }
    Ok(read_len)
}
