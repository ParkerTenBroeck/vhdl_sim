use std::{
    io::{BufRead, BufReader},
    sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering},
};

#[repr(u8)]
pub enum Sig {
    SU = 0,
    SX = 1,
    S0 = 2,
    S1 = 3,
    SZ = 4,
    SW = 5,
    SL = 6,
    SH = 7,
    SD = 8
}

pub struct SimState{
    switch: AtomicU32,
    button: AtomicU32,
    led: AtomicU32,
    segs: [AtomicU32; 4],
    updated: AtomicU8,
}

static STATE: SimState = SimState{
    switch: AtomicU32::new(512),
    button: AtomicU32::new(0),
    led: AtomicU32::new(0),
    segs: [const{AtomicU32::new(0)}; 4],
    updated: AtomicU8::new(0)
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
        } else if let value = STATE.updated.swap(0, Ordering::Relaxed) && value!=0 {
            if (value >> 0) & 1 == 1{
                eprintln!("led={}", STATE.led.load(Ordering::Relaxed));
            }
            for i in 0..4{
                if (value >> (i+1)) & 1 == 1{
                    eprintln!("seg{i}={}", STATE.segs[i].load(Ordering::Relaxed));
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_init() {
    std::thread::Builder::new().name("client".into()).spawn(client).expect("Failed to spawn client thread");
    for arg in std::env::args(){
        if let Some(arg) = arg.strip_prefix("--cycle_sleep").or(arg.strip_prefix("-c")){
            if let Ok(nanos) = arg.trim().parse::<u64>(){
                SLEEP_NANOS.store(nanos, Ordering::Relaxed);
            }else{
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
pub extern "C" fn ffi_set_outputs(led: u32, seg0: u32, seg1: u32, seg2: u32, seg3: u32) {
    let o_led = STATE.led.swap(led, Ordering::Relaxed) != led;

    let o_seg0 = STATE.segs[0].swap(seg0, Ordering::Relaxed) != seg0;
    let o_seg1 = STATE.segs[1].swap(seg1, Ordering::Relaxed) != seg1;
    let o_seg2 = STATE.segs[2].swap(seg2, Ordering::Relaxed) != seg2;
    let o_seg3 = STATE.segs[3].swap(seg3, Ordering::Relaxed) != seg3;

    let to_set = (o_led as u8) << 0
                        | (o_seg0 as u8) << 1
                        | (o_seg1 as u8) << 2
                        | (o_seg2 as u8) << 3
                        | (o_seg3 as u8) << 4;

    STATE.updated.fetch_or(to_set, Ordering::Relaxed);
    
    std::thread::sleep(std::time::Duration::from_nanos(SLEEP_NANOS.load(Ordering::Relaxed)));
}

