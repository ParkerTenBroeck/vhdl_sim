use std::{
    io::{BufRead, BufReader},
    sync::atomic::{AtomicU32, Ordering},
};

pub struct SimState{
    switch: AtomicU32,
    button: AtomicU32,
    led: AtomicU32,
    hex: AtomicU32,
}

static STATE: SimState = SimState{
    switch: AtomicU32::new(0),
    button: AtomicU32::new(0),
    led: AtomicU32::new(0),
    hex: AtomicU32::new(0),
};

fn client() {
    let reader = BufReader::new(std::io::stdin());
    for line in reader.lines().map_while(Result::ok) {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("sw=") {
            if let Ok(n) = v.parse::<u32>() {
                STATE.switch.store(n, Ordering::Relaxed);
            }
        } else if let Some(v) = line.strip_prefix("key=") {
            if let Ok(n) = v.parse::<u32>() {
                STATE.button.store(n, Ordering::Relaxed);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn ffi_init() {
    std::thread::Builder::new().name("client".into()).spawn(client).expect("Failed to spawn client thread");
    eprintln!("[ffi] initialzied");
}

#[no_mangle]
pub extern "C" fn ffi_get_sw() -> u32 {
    STATE.switch.load(Ordering::Relaxed)
}

#[no_mangle]
pub extern "C" fn ffi_get_key() -> u32 {
    STATE.button.load(Ordering::Relaxed)
}

#[no_mangle]
pub extern "C" fn ffi_set_outputs(led: u32, hex: u32) {
    if STATE.led.swap(led, Ordering::Relaxed) != led{
        println!("LED={:#x?}", STATE.led.load(Ordering::Relaxed))
    }
    if STATE.hex.swap(hex, Ordering::Relaxed) != hex{
        println!("HEX={:#x?}", STATE.hex.load(Ordering::Relaxed))
    }
}