//
// Copyright 2024 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

#![no_std]
#![no_main]

use core::{
    ops::{Index, IndexMut},
    panic::PanicInfo,
};

use log::info;
use oak_stage0::paging;
use oak_tdx_guest::{
    tdcall::get_td_info,
    vmcall::{
        call_cpuid, io_read_u16, io_read_u32, io_read_u8, io_write_u16, io_write_u32, io_write_u8,
        mmio_read_u32, mmio_write_u32, msr_read, msr_write,
    },
};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        OffsetPageTable, Page, PageSize, PageTable, PageTableIndex, Size1GiB, Size2MiB, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

mod asm;

#[no_mangle]
static mut GPAW: u32 = 0;

#[no_mangle]
static mut AP_IN_64BIT_COUNT: u32 = 0;

static HELLO_OAK: &str = "Hello from stage0_bin_tdx!";

fn get_tdx_shared_bit() -> usize {
    unsafe { GPAW as usize - 1 }
}

// Inspired by TD-shim and credits to TD-shim
fn offset_pt() -> OffsetPageTable<'static> {
    let cr3 = Cr3::read().0.start_address().as_u64();
    unsafe { OffsetPageTable::new(&mut *(cr3 as *mut _), VirtAddr::new(0)) }
}

fn pt_entry_set_shared_bit(page_table: &mut PageTable, index: PageTableIndex, shared: bool) {
    let entry = page_table.index(index);
    let shared_bit = 1 << get_tdx_shared_bit();

    let addr = if shared {
        entry.addr().as_u64() | shared_bit
    } else {
        entry.addr().as_u64() & !shared_bit
    };
    let flags = entry.flags();

    page_table.index_mut(index).set_addr(PhysAddr::new(addr), flags);
}

// TODO: b/360129756 - simplify this function. consider merging it into stage0
fn pt_set_shared_bit(pt: &mut OffsetPageTable, page: &Page, shared: bool) {
    let p4 = pt.level_4_table();
    let p3 = unsafe { &mut *(p4.index(page.p4_index()).addr().as_u64() as *mut PageTable) };

    if page.size() == Size1GiB::SIZE {
        pt_entry_set_shared_bit(p3, page.p3_index(), shared);
    }

    let p2 = unsafe { &mut *(p3.index(page.p3_index()).addr().as_u64() as *mut PageTable) };
    if page.size() == Size2MiB::SIZE {
        pt_entry_set_shared_bit(p2, page.p2_index(), shared);
    }

    let p1 = unsafe { &mut *(p2.index(page.p2_index()).addr().as_u64() as *mut PageTable) };
    if page.size() == Size4KiB::SIZE {
        pt_entry_set_shared_bit(p1, page.p1_index(), shared);
    }
}

fn write_u8_to_serial(c: u8) {
    // wait_for_empty_output
    loop {
        if (io_read_u8(0x3f8 + 0x5).unwrap() & (1 << 5)) != 0 {
            break;
        }
    }
    io_write_u8(0x3f8, c).unwrap();
}

fn write_single_hex(c: u8) {
    if c < 0xa {
        write_u8_to_serial(c + (b'0'));
    } else {
        write_u8_to_serial(c - 10 + (b'a'));
    }
}

fn write_byte_hex(c: u8) {
    let char1 = (c >> 4) & 0xF;
    let char2 = c & 0xF;
    write_single_hex(char1);
    write_single_hex(char2);
}

fn write_u32(n: u32) {
    let b = n.to_le_bytes();
    for c in b.iter().rev() {
        write_byte_hex(*c);
    }
    write_u8_to_serial(b'\n');
}

fn write_u64(n: u64) {
    let b = n.to_le_bytes();
    for c in b.iter().rev() {
        write_byte_hex(*c);
    }
    write_u8_to_serial(b'\n');
}

fn write_str(s: &str) {
    for c in s.bytes() {
        write_u8_to_serial(c);
    }
    write_u8_to_serial(b'\n');
}

fn debug_u32_variable(s: &str, val: u32) {
    for c in s.bytes() {
        write_u8_to_serial(c);
    }
    write_u8_to_serial(b':');
    write_u8_to_serial(b' ');
    write_u32(val);
}

fn debug_u64_variable(s: &str, val: u64) {
    for c in s.bytes() {
        write_u8_to_serial(c);
    }
    write_u8_to_serial(b':');
    write_u8_to_serial(b' ');
    write_u64(val);
}

fn init_tdx_serial_port() {
    io_write_u8(0x3f8 + 0x1, 0x0).unwrap(); // Disable interrupts
    io_write_u8(0x3f8 + 0x2, 0x0).unwrap(); // Disable FIFO
    io_write_u8(0x3f8 + 0x3, 0x3).unwrap(); // LINE_CONTROL_8N1
    io_write_u8(0x3f8 + 0x4, 0x3).unwrap(); // DATA_TERMINAL_READY_AND_REQUEST_TO_SEND
}

use oak_stage0::hal::PortFactory;

struct Mmio {}
impl<S: x86_64::structures::paging::page::PageSize> oak_stage0::hal::Mmio<S> for Mmio {
    fn read_u32(&self, offset: usize) -> u32 {
        mmio_read_u32(offset as *const u32).unwrap()
    }
    unsafe fn write_u32(&mut self, offset: usize, val: u32) {
        mmio_write_u32(offset as *mut u32, val).unwrap()
    }
}

struct Tdx {}
impl oak_stage0::Platform for Tdx {
    type Mmio<S: x86_64::structures::paging::page::PageSize> = Mmio;
    fn cpuid(leaf: u32) -> core::arch::x86_64::CpuidResult {
        call_cpuid(leaf, 0).unwrap()
    }

    unsafe fn mmio<S>(_: x86_64::addr::PhysAddr) -> <Self as oak_stage0::Platform>::Mmio<S>
    where
        S: x86_64::structures::paging::page::PageSize,
    {
        todo!()
    }

    fn port_factory() -> PortFactory {
        PortFactory {
            read_u8: |port| io_read_u8(port as u32),
            read_u16: |port| io_read_u16(port as u32),
            read_u32: |port| io_read_u32(port as u32),
            write_u8: |port, val| io_write_u8(port as u32, val),
            write_u16: |port, val| io_write_u16(port as u32, val),
            write_u32: |port, val| io_write_u32(port as u32, val),
        }
    }

    fn early_initialize_platform() {
        write_str("early_initialize_platform");
        write_str("early_initialize_platform completed");
    }
    fn initialize_platform(e820_table: &[oak_linux_boot_params::BootE820Entry]) {
        // logger is initialized starting from here
        info!("initialize platform");
        info!("{:?}", e820_table);
        info!("initialize platform completed");
    }
    fn deinit_platform() {
        todo!()
    }
    fn populate_zero_page(_: &mut oak_stage0::ZeroPage) {
        todo!()
    }
    fn get_attestation(
        _: [u8; 64],
    ) -> Result<oak_sev_snp_attestation_report::AttestationReport, &'static str> {
        todo!()
    }
    fn get_derived_key() -> Result<[u8; 32], &'static str> {
        todo!()
    }
    fn change_page_state(
        page: x86_64::structures::paging::Page,
        attr: oak_stage0::hal::PageAssignment,
    ) {
        let shared: bool = match attr {
            oak_stage0::hal::PageAssignment::Shared => true,
            oak_stage0::hal::PageAssignment::Private => false,
        };
        let mut pt = offset_pt();
        pt_set_shared_bit(&mut pt, &page, shared);
    }
    fn revalidate_page(_: x86_64::structures::paging::Page) {
        todo!()
    }
    fn page_table_mask(enc: oak_stage0::paging::PageEncryption) -> u64 {
        // a. When 4-level EPT is active, the SHARED bit position would
        // always be bit 47.
        // b. When 5-level EPT is active, the SHARED bit position would
        // be bit 47 if GPAW is 0. Otherwise, else it would be bit 51.
        match enc {
            oak_stage0::paging::PageEncryption::Encrypted => 0,
            oak_stage0::paging::PageEncryption::Unencrypted => 1 << get_tdx_shared_bit(),
        }
    }
    fn encrypted() -> u64 {
        // stage0_bin_tdx does not support regular VM
        1 << get_tdx_shared_bit()
    }
    fn tee_platform() -> oak_dice::evidence::TeePlatform {
        todo!()
    }
    unsafe fn read_msr(msr: u32) -> u64 {
        msr_read(msr).unwrap()
    }
    unsafe fn write_msr(msr: u32, value: u64) {
        msr_write(msr, value).unwrap()
    }
}

/// Entry point for the Rust code in the stage0 BIOS.
#[no_mangle]
pub extern "C" fn rust64_start() -> ! {
    init_tdx_serial_port();
    write_str(HELLO_OAK);
    debug_u32_variable(stringify!(GPAW), unsafe { GPAW });
    assert!(unsafe { GPAW == 48 || GPAW == 52 });

    let td_info = get_td_info();
    debug_u64_variable(stringify!(td_info.gpa_width), td_info.gpa_width as u64);
    debug_u64_variable(stringify!(td_info.attributes), td_info.attributes.bits() as u64);
    debug_u32_variable(stringify!(td_info.max_vcpus), td_info.max_vcpus);
    debug_u32_variable(stringify!(td_info.num_vcpus), td_info.num_vcpus);
    debug_u32_variable(stringify!(AP_IN_64BIT_COUNT), unsafe { AP_IN_64BIT_COUNT });

    oak_stage0::rust64_start::<Tdx>()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    oak_stage0::panic(info)
}
