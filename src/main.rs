extern crate systemstat;

use chrono::{Datelike, Local, Utc};
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::thread;
use std::time::Duration;
use systemstat::{Platform, System};

struct Stat {
    value: String,
    fetch: Box<dyn Fn() -> String>,
    interval: u8,
}
impl Stat {
    fn new(fetch: impl Fn() -> String + 'static, interval: u8) -> Self {
        Stat {
            value: String::new(),
            fetch: Box::new(fetch),
            interval,
        }
    }
    fn set(&mut self, val: String) {
        self.value = val;
    }
}

fn setbar(s: String) {
    Command::new("xsetroot")
        .arg("-name")
        .arg(s)
        .output()
        .expect("failed to execute process");
}

fn main() {
    //i really dont know how rust works
    //let sys: System = System::new();

    let mut stats: Vec<Stat> = vec![
        Stat::new(
            //MOUNT
            move || match System::new().mount_at("/") {
                Ok(mount) => {
                    let s = mount.avail.to_string();
                    return format!("(MNT:{}G)", s[..s.len() - 3].to_string());
                }
                Err(_) => String::from("(MNT)"),
            },
            30,
        ),
        Stat::new(move || String::from(" "), 0),
        Stat::new(
            //RAM
            move || match System::new().memory() {
                Ok(mem) => format!(
                    "<RAM:{}",
                    (mem.total.as_u64() - mem.free.as_u64()) * 100 / mem.total.as_u64()
                ),
                Err(_) => String::from("<RAM>"),
            },
            2,
        ),
        Stat::new(
            //SWAP
            move || match System::new().swap() {
                Ok(swap) => format!(
                    ":{}%>",
                    (swap.total.as_u64() - swap.free.as_u64()) * 100 / swap.total.as_u64()
                ),
                Err(_) => String::from("%>"),
            },
            4,
        ),
        Stat::new(move || String::from(" {"), 0),
        Stat::new(
            //CPU USAGE + TEMPERATURE
            move || match System::new().cpu_load_aggregate() {
                Ok(cpu) => {
                    thread::sleep(Duration::from_secs(1));
                    return format!(
                        "CPU:{}%{}°",
                        ((1.0 - cpu.done().unwrap().idle) * 100.0).round(),
                        String::from_utf8_lossy(
                            &Command::new("sh")
                                .arg("-c")
                                .arg("sensors | grep Tctl: | awk '{print int($2);}'")
                                .output()
                                .expect("0")
                                .stdout
                        )
                        .trim()
                    );
                }
                Err(_) => String::from("CPU"),
            },
            2,
        ),
        Stat::new(move || String::from(" "), 0),
        Stat::new(
            //GPU USAGE + TEMPERATURE
            move || {
                format!(
                    "GPU:{}%{}°",
                    String::from_utf8_lossy(
                        &Command::new("nvidia-smi")
                            .arg("--format=csv,noheader,nounits")
                            .arg("--query-gpu=utilization.gpu")
                            .output()
                            .expect("GPU")
                            .stdout
                    )
                    .trim(),
                    String::from_utf8_lossy(
                        &Command::new("nvidia-smi")
                            .arg("--format=csv,noheader,nounits")
                            .arg("--query-gpu=temperature.gpu")
                            .output()
                            .expect("GPU")
                            .stdout
                    )
                    .trim()
                )
            },
            2,
        ),
        Stat::new(move || String::from("} "), 0),
        Stat::new(
            //DATE & TIME
            move || {
                let t = Local::now();
                return format!(
                    "[{}.{}]",
                    t.weekday().num_days_from_sunday(),
                    t.format("%d/%m|%H:%M")
                );
            },
            1,
        ),
        Stat::new(move || String::from(" \\"), 0),
        Stat::new(
            //BATTERY
            move || {
                let sys = System::new();
                /*let ac;
                match sys.on_ac_power() {
                    Ok(power) => ac = power,
                    Err(_) => {}
                }*/
                match sys.battery_life() {
                    Ok(battery) => {
                        if battery.remaining_capacity > 0.0 {
                            if battery.remaining_capacity > 0.1 {
                                if battery.remaining_capacity > 0.2 {
                                    if battery.remaining_capacity > 0.3 {
                                        if battery.remaining_capacity > 0.4 {
                                            if battery.remaining_capacity > 0.5 {
                                                if battery.remaining_capacity > 0.6 {
                                                    if battery.remaining_capacity > 0.7 {
                                                        if battery.remaining_capacity > 0.8 {
                                                            if battery.remaining_capacity > 0.9 {
                                                                return String::from("󰁹");
                                                            }
                                                            return String::from("󰂂");
                                                        }
                                                        return String::from("󰂁");
                                                    }
                                                    return String::from("󰂀");
                                                }
                                                return String::from("󰁿");
                                            }
                                            return String::from("󰁾");
                                        }
                                        return String::from("󰁽");
                                    }
                                    return String::from("󰁼");
                                }
                                return String::from("󰁻");
                            }
                            return String::from("󰁺");
                        } else {
                            return String::from("󰂎");
                        }
                    }
                    Err(_) => String::from("󰂎"),
                }
            },
            10,
        ),
        Stat::new(move || String::from("|"), 0),
        Stat::new(
            //VOLUME
            move || {
                String::from_utf8_lossy(&Command::new("sh")
                .arg("-c")
                .arg("amixer sget Master | awk -F\"[][]\" '/Left:/ { gsub(\"%\",\"\"); if($4==\"on\"){ if($2 <= 25) print \"󰕿\"; else if($2 <= 75) print \"󰖀\"; else if($2<=100) print \"󰕾\"; } else print \"󰝟\"; }'")
                .output()
                .expect("󰝟").stdout).trim().to_string()
            },
            10,
        ),
        Stat::new(
            //BRIGHTNESS
            move || {
                String::from_utf8_lossy(&Command::new("sh")
                .arg("-c")
                .arg("xbacklight -get | awk '{ if($1 <= 10) print \"󱩎\"; else if($1 <= 20) print \"󱩏\"; else if($1<=30) print \"󱩐\"; else if($1<=40) print \"󱩑\"; else if($1<=50) print \"󱩒\"; else if($1<=60) print \"󱩓\"; else if($1<=70) print \"󱩔\"; else if($1<=80) print \"󱩕\"; else if($1<=90) print \"󱩖\"; else if($1<=100) print \"󰛨\"; }'")
                .output()
                .expect("󰛨").stdout).trim().to_string()
            },
            10,
        ),
        Stat::new(move || String::from("|"), 0),
        Stat::new(
            //WIFI
            move || match File::open("/proc/net/wireless") {
                Ok(mut file) => {
                    let mut contents = String::new();
                    let _ = file.read_to_string(&mut contents);
                    let s = contents.split("\n").collect::<Vec<_>>()[2];
                    if s.len() < 3 {
                        return String::from("󰤯");
                    };
                    let s = s.split_whitespace().collect::<Vec<_>>()[2];
                    let num: Result<u8, _> = s[..s.len() - 1].parse();
                    match num {
                        Ok(e) => {
                            if e > 0 {
                                if e > 23 {
                                    if e > 46 {
                                        return String::from("󰤥");
                                    } else {
                                        return String::from("󰤢");
                                    }
                                } else {
                                    return String::from("󰤟");
                                }
                            } else {
                                return String::from("󰤯");
                            }
                        }
                        Err(_) => {}
                    }
                    String::from("󰤯")
                }
                Err(_) => String::from("󰤯"),
            },
            5,
        ),
        /*Stat::new( //BLUETOOTH
            move || {
                String::from("")
            },
            5,
        ),*/
        Stat::new(move || String::from("|"), 0),
        Stat::new(
            //KEYBOARD
            move || {
                String::from_utf8_lossy(
                    &Command::new("sh")
                        .arg("-c")
                        .arg("xkb-switch | cut -d '(' -f 1")
                        .output()
                        .expect("un")
                        .stdout,
                )
                .trim()
                .to_string()
            },
            10,
        ),
        Stat::new(move || String::from(" \\"), 0),
        Stat::new(
            move || {
                format!(
                    "{}@{}",
                    String::from_utf8_lossy(&Command::new("whoami").output().expect("").stdout)
                        .trim(),
                    String::from_utf8_lossy(
                        &Command::new("uname").arg("-n").output().expect("").stdout
                    )
                    .trim()
                )
            },
            0,
        ),
    ];

    for stat in &mut stats {
        if stat.interval == 0 {
            stat.set((stat.fetch)());
        }
    }
    let mut t = 0;
    let mut i: u8 = 0;
    loop {
        if t + 2 > Utc::now().timestamp() {
            continue;
        }
        t = Utc::now().timestamp();

        for stat in &mut stats {
            if stat.interval > 0 && i % stat.interval == 0 {
                stat.set((stat.fetch)());
            }
        }
        i += 1;
        let f: Vec<String> = stats.iter().map(|x| format!("{}", x.value)).collect();
        setbar(f.join(""));
        //thread::sleep(Duration::from_secs(1));//replace with time continue
    }
}
