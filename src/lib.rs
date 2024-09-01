#![no_std]

mod addr;
mod config;
mod frame;
mod page_table;
mod page_table_entry;
mod pte_flags;

extern crate alloc;

pub use crate::page_table_entry::PageTableEntry;
pub use crate::pte_flags::PTEFlags;
pub use addr::*;
use alloc::string::String;
use alloc::vec::Vec;
pub use config::*;
pub use frame::*;
pub use page_table::*;

pub fn init_frame_allocator(from: usize, to: usize) {
    FRAME_ALLOCATOR
        .exclusive_access()
        .init(PhysAddr::from(from).ceil(), PhysAddr::from(to).floor());
}

/// translate a pointer to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();

    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));

        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

/// translate a pointer to a mutable u8 Vec end with `\0` through page table to a `String`
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();

    let mut va = ptr as usize;
    loop {
        let ch: u8 = {
            let pa = page_table.translate_va(VirtAddr::from(va)).unwrap();
            let pa: usize = pa.into();
            let pa = pa as *const u8;
            unsafe { *pa }
        };

        if ch == 0 {
            break;
        } else {
            string.push(ch as char);
            va += 1;
        }
    }

    string
}

pub fn translate_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;

    page_table
        .translate_va(VirtAddr::from(va))
        .unwrap()
        .get_mut()
}
