use std::{
    io::{BufRead, BufReader},
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

pub struct SimState {
    switch: AtomicU32,
    button: AtomicU32,
    led: AtomicU32,
    /// represents 4 segments (each byte being one segment)
    segs: [AtomicU32; 8],
    updated: AtomicU32,
}

static STATE: SimState = SimState {
    switch: AtomicU32::new(0),
    button: AtomicU32::new(0),
    led: AtomicU32::new(0),
    segs: [const { AtomicU32::new(0) }; 8],
    updated: AtomicU32::new(0),
};

/// default 1 ms
static SLEEP_NANOS: AtomicU64 = AtomicU64::new(1_000_000);

fn client() {
    let reader = BufReader::new(std::io::stdin());
    for line in reader.lines().map_while(Result::ok) {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("sw=") {
            if let Ok(n) = v.parse::<u32>() {
                STATE.switch.store(n, Ordering::Relaxed);
            }
        } else if let Some(v) = line.strip_prefix("btn=") {
            if let Ok(n) = v.parse::<u32>() {
                STATE.button.store(n, Ordering::Relaxed);
            }
        } else if let value = STATE.updated.swap(0, Ordering::Relaxed)
            && value != 0
        {
            if (value >> 0) & 1 == 1 {
                eprintln!("led={}", STATE.led.load(Ordering::Relaxed));
            }
            for i in 0..STATE.segs.len() {
                if (value >> (i + 1)) & 1 == 1 {
                    eprintln!("seg={};{i}", STATE.segs[i].load(Ordering::Relaxed));
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_init() {
    std::thread::Builder::new()
        .name("client".into())
        .spawn(client)
        .expect("Failed to spawn client thread");
    for arg in std::env::args() {
        if let Some(arg) = arg.strip_prefix("--cycle_sleep").or(arg.strip_prefix("-c")) {
            if let Ok(nanos) = arg.trim().parse::<u64>() {
                SLEEP_NANOS.store(nanos, Ordering::Relaxed);
            } else {
                eprintln!("cycle sleep(ns) failed to parse");
            }
        }
    }
    eprintln!("[ffi] initialzied");
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_get_sw() -> u32 {
    STATE.switch.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_get_btn() -> u32 {
    STATE.button.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_set_outputs(led: u32, segv: u32, segs: u32) {
    let o_led = STATE.led.swap(led, Ordering::Relaxed) != led;

    let mut to_set = o_led as u32;

    if let Some(place) = STATE.segs.get(segs as usize) {
        let seg = place.swap(segv, Ordering::Relaxed) != segv;
        to_set |= (seg as u32) << (segs+1)
    }

    STATE.updated.fetch_or(to_set, Ordering::Relaxed);

    std::thread::sleep(std::time::Duration::from_nanos(
        SLEEP_NANOS.load(Ordering::Relaxed),
    ));
}
