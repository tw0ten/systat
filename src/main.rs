use std::{process::Command, thread, time::Duration};
use systemstat::{Platform, System};

mod config;
mod stat;

static mut S: i16 = 0;
const INTERVAL: Duration = Duration::from_millis(500);

fn setbar(s: &str) {
    _ = Command::new("xsetroot").arg("-name").arg(s).status()
}

fn main() {
    setbar("systat");
    let mut stats = config::get();
    let sys = System::new();
    for stat in &mut stats {
        if stat.i > 0 {
            continue;
        }
        stat.f(&sys)
    }

    let mut i: i16 = 0;
    loop {
        let sig = unsafe { S };
        unsafe { S = 0 }
        let mut s = String::new();
        for stat in &mut stats {
            if match stat.i {
                0 => false,
                0.. => i % stat.i == 0,
                _ => stat.i == sig,
            } {
                stat.f(&sys);
            }
            s.push_str(&stat.s)
        }
        setbar(&s);
        i = i.wrapping_add(1);
        thread::sleep(INTERVAL)
    }
}
