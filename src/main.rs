extern crate systemstat;

use chrono::{Datelike, Local, Utc};
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::thread;
use std::time::Duration;
use systemstat::{Platform, System};

mod stat;
use stat::Stat;

fn setbar(s: String) {
    Command::new("xsetroot")
        .arg("-name")
        .arg(s)
        .output()
        .expect("failed to $xsetroot -name \"...\"");
}

fn main() {
    let sys: System = System::new();

    let mut stats: Vec<Stat> = vec![
        Stat::new(
            //MOUNT
            |sys| match sys.mount_at("/") {
                Ok(mount) => {
                    let s = mount.avail.to_string();
                    return format!("(MNT:{}G)", s[..s.len() - 3].to_string());
                }
                Err(_) => String::from("(MNT)"),
            },
            30,
        ),
        Stat::new(|_| String::from(" "), 0),
        Stat::new(
            //RAM
            |sys| match sys.memory() {
                Ok(mem) => format!(
                    "<RAM:{}",
                    (mem.total.as_u64() - mem.free.as_u64()) * 100 / mem.total.as_u64()
                ),
                Err(_) => String::from("<RAM:-"),
            },
            2,
        ),
        Stat::new(
            //SWAP
            |sys| match sys.swap() {
                Ok(swap) => format!(
                    ":{}%>",
                    (swap.total.as_u64() - swap.free.as_u64()) * 100 / swap.total.as_u64()
                ),
                Err(_) => String::from("%>"),
            },
            4,
        ),
        Stat::new(|_| String::from(" {"), 0),
        Stat::new(
            //CPU USAGE + TEMPERATURE
            |sys| match sys.cpu_load_aggregate() {
                Ok(cpu) => {
                    thread::sleep(Duration::from_millis(500));
                    return format!(
                        "CPU:{}%{}°",
                        ((1.0 - cpu.done().unwrap().idle) * 100.0).round(),
                        match Command::new("sh")
                            .arg("-c")
                            .arg("sensors | grep Tctl: | awk '{print int($2);}'")
                            .output()
                        {
                            Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                            Err(_) => String::from("-"),
                        }
                    );
                }
                Err(_) => String::from("CPU"),
            },
            2,
        ),
        Stat::new(|_| String::from(" "), 0),
        Stat::new(
            //GPU USAGE + TEMPERATURE
            |_| {
                format!(
                    "GPU:{}%{}°",
                    match Command::new("nvidia-smi")
                        .arg("--format=csv,noheader,nounits")
                        .arg("--query-gpu=utilization.gpu")
                        .output()
                    {
                        Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                        Err(_) => String::from("-"),
                    },
                    match Command::new("nvidia-smi")
                        .arg("--format=csv,noheader,nounits")
                        .arg("--query-gpu=temperature.gpu")
                        .output()
                    {
                        Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                        Err(_) => String::from("-"),
                    }
                )
            },
            2,
        ),
        Stat::new(|_| String::from("} "), 0),
        Stat::new(
            //DATE & TIME
            |_| {
                let t = Local::now();
                return format!(
                    "[{}.{}]",
                    t.weekday().num_days_from_sunday(),
                    t.format("%d/%m|%H:%M")
                );
            },
            1,
        ),
        Stat::new(|_| String::from(" \\"), 0),
        Stat::new(
            //BATTERY
            |sys| match sys.battery_life() {
                Ok(battery) => {
                    if battery.remaining_capacity > 0.9 {
                        return String::from("󰁹");
                    }
                    if battery.remaining_capacity > 0.8 {
                        return String::from("󰂂");
                    }
                    if battery.remaining_capacity > 0.7 {
                        return String::from("󰂁");
                    }
                    if battery.remaining_capacity > 0.6 {
                        return String::from("󰂀");
                    }
                    if battery.remaining_capacity > 0.5 {
                        return String::from("󰁿");
                    }
                    if battery.remaining_capacity > 0.4 {
                        return String::from("󰁾");
                    }
                    if battery.remaining_capacity > 0.3 {
                        return String::from("󰁽");
                    }
                    if battery.remaining_capacity > 0.2 {
                        return String::from("󰁼");
                    }
                    if battery.remaining_capacity > 0.1 {
                        return String::from("󰁻");
                    }
                    if battery.remaining_capacity > 0.05 {
                        return String::from("󰁺");
                    }
                    String::from("󰂎")
                }
                Err(_) => String::from("󰂎"),
            },
            10,
        ),
        Stat::new(
            //AC
            |sys| match sys.on_ac_power() {
                Ok(power) => {
                    if power {
                        return String::new();
                    }
                    String::from("-")
                }
                Err(_) => String::from("?"),
            },
            5,
        ),
        Stat::new(|_| String::from("|"), 0),
        Stat::new(
            //VOLUME
            |_| {
                match Command::new("sh")
                    .arg("-c")
                    .arg("amixer sget Master | awk -F\"[][]\" '/Left:/ { gsub(\"%\",\"\"); if($4==\"on\"){ if($2 <= 25) print \"󰕿\"; else if($2 <= 75) print \"󰖀\"; else if($2<=100) print \"󰕾\"; } else print \"󰝟\"; }'")
                    .output(){
                    Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                    Err(_) => String::from("󰝟")
                }
            },
            10,
        ),
        Stat::new(
            //BRIGHTNESS
            |_| {
                match Command::new("sh")
                    .arg("-c")
                    .arg("xbacklight -get | awk '{ split($0, o, \".\"); print o[1]; }'")
                    .output(){
                    Ok(s) => {
                        let n: u8 = String::from_utf8_lossy(&s.stdout).trim().to_string().parse().expect("failed to parse brightness value");
                        if n > 90 {
                            return String::from("󰛨");
                        }
                        if n > 80 {
                            return String::from("󱩖");
                        }
                        if n > 70 {
                            return String::from("󱩕");
                        }
                        if n > 60 {
                            return String::from("󱩔");
                        }
                        if n > 50 {
                            return String::from("󱩓");
                        }
                        if n > 40 {
                            return String::from("󱩒");
                        }
                        if n > 30 {
                            return String::from("󱩑");
                        }
                        if n > 20 {
                            return String::from("󱩐");
                        }
                        if n > 10 {
                            return String::from("󱩏");
                        }
                        String::from("󱩎")
                    },
                    Err(_) => String::from("󱩎")
                }
            },
            10,
        ),
        Stat::new(|_| String::from("|"), 0),
        Stat::new(
            //WIFI
            |_| match File::open("/proc/net/wireless") {
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
                            if e > 50 {
                                return String::from("󰤨");
                            }
                            if e > 33 {
                                return String::from("󰤥");
                            }
                            if e > 16 {
                                return String::from("󰤢");
                            }
                            if e > 0 {
                                return String::from("󰤟");
                            }
                            String::from("󰤯")
                        }
                        Err(_) => String::from("󰤯"),
                    }
                }
                Err(_) => String::from("󰤯"),
            },
            5,
        ),
        // Stat::new(
        //     //BLUETOOTH
        //     |_| match Command::new("sh")
        //         .arg("-c")
        //         .arg(
        //             "rfkill list bluetooth | grep -qo \"Soft blocked: no\" && echo '󰂯' || echo '󰂲'",
        //         )
        //         .output()
        //     {
        //         Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
        //         Err(_) => String::new(),
        //     },
        //     10,
        // ),
        Stat::new(|_| String::from("|"), 0),
        Stat::new(
            //KEYBOARD
            |_| match Command::new("sh")
                .arg("-c")
                .arg("xkb-switch | cut -d '(' -f 1")
                .output()
            {
                Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                Err(_) => String::from("--"),
            },
            10,
        ),
        Stat::new(|_| String::from(" \\"), 0),
        Stat::new(
            //USER@HOST
            |_| {
                format!(
                    "{}@{}",
                    match Command::new("whoami").output() {
                        Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                        Err(_) => String::from("user"),
                    },
                    match Command::new("uname").arg("-n").output() {
                        Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                        Err(_) => String::from("host"),
                    }
                )
            },
            0,
        ),
    ];

    let mut t: i64 = 0;
    let mut i: u8 = 0;

    //fetch one time Stats
    for stat in &mut stats {
        if stat.i <= 0 {
            stat.fetch(&sys);
        }
    }

    loop {
        if t + 1 > Utc::now().timestamp() {
            continue;
        }
        t = Utc::now().timestamp();

        let mut s: String = String::new();
        for stat in &mut stats {
            if stat.i > 0 && i % stat.i == 0 {
                stat.fetch(&sys);
            }
            s += &stat.s;
        }
        i += 1;
        setbar(s);
    }
}
