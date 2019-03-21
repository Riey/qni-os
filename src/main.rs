#![no_std]
#![no_main]
#![feature(alloc)]

#[macro_use]
extern crate alloc;

use core::mem;
use alloc::string::String;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::table::boot::{
    MemoryDescriptor,
    SearchType,
};
use uefi::{
    Handle,
};
use uefi::data_types::Align;
use uefi::proto::media::{
    fs::SimpleFileSystem,
    file::*,
};

fn shutdown(image: Handle, st: SystemTable<Boot>) -> ! {
    use uefi::table::runtime::ResetType;

    log::info!("shutdown...");

    let max_mmap_size =
        st.boot_services().memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();

    let mut mmap_storage = vec![0; max_mmap_size].into_boxed_slice();

    let (st, _iter) = st
        .exit_boot_services(image, &mut mmap_storage[..])
        .expect_success("Failed to exit boot services");

    let rt = unsafe { st.runtime_services() };

    rt.reset(ResetType::Shutdown, Status::SUCCESS, None);
}

fn app_main(image: Handle, st: SystemTable<Boot>) -> uefi::Result<Status> {

    let size = st
        .boot_services()
        .locate_handle(SearchType::from_proto::<SimpleFileSystem>(), None)?
        .unwrap();

    log::info!("Get {} handles for SimpleFileSystem", size);

    assert!(size > 0);

    let mut buffer = Vec::with_capacity(size);

    unsafe { buffer.set_len(size); }

    st.boot_services()
        .locate_handle(SearchType::from_proto::<SimpleFileSystem>(), Some(&mut buffer[..]))?
        .unwrap();

    let fs_handle = buffer.first().unwrap();

    let fs = st.boot_services()
        .handle_protocol::<SimpleFileSystem>(*fs_handle)?
        .unwrap()
        .get();
        
    let mut root = unsafe { (*fs).open_volume()?.unwrap() };

    let mut buf = vec![0; FileInfo::alignment() * 50];
    let root_file = loop {
        match root.read_entry(&mut buf[..]).map_err(|err| err.split()) {
            Ok(ret) => break ret.log().unwrap(),
            Err((_, Some(new_size))) => {
                buf.extend((0..new_size - buf.len()).map(|_| 0));
            }
            Err((status, None)) => panic!("Can't read root dir status: {:?}", status),
        };
    };

    log::info!("name: {:?}", String::from_utf16_lossy(root_file.file_name().to_u16_slice()));

    

    shutdown(image, st);
}

#[no_mangle]
pub extern "win64" fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");

    log::info!("Entering efi_main...");

    app_main(image, st)
        .unwrap_success()
}

