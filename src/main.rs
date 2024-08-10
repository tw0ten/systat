use chrono::{Datelike, Local};
use std::{fs::File, io::Read, process::Command};
use std::{thread::sleep, time::Duration};
use systemstat::{Platform, System};

mod stat;
use stat::Stat;

pub static mut S: i8 = 0;

fn main() {
    let sys = System::new();
    let mut stats = vec![
        Stat::new(
            //MOUNT
            |sys| match sys.mount_at("/") {
                Ok(mount) => {
                    let s = mount.avail.to_string();
                    return format!("(MNT:{}G)", s[..s.len() - 3].to_string());
                }
                _ => String::from("(MNT)"),
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
                _ => String::from("<RAM:-"),
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
                _ => String::from("%>"),
            },
            4,
        ),
        Stat::new(|_| String::from(" {"), 0),
        Stat::new(
            //CPU USAGE + TEMPERATURE
            |sys| match sys.cpu_load_aggregate() {
                Ok(cpu) => {
                    sleep(Duration::from_millis(500));
                    return format!(
                        "CPU:{}%{}°",
                        ((1.0 - cpu.done().unwrap().idle) * 100.0).round(),
                        match Command::new("sh")
                            .arg("-c")
                            .arg("sensors | grep ^Composite: | sed 's/\\..*//' | sed 's/Composite:.*[+-]//'")
                            .output()
                        {
                            Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                            _ => String::from("-"),
                        }
                    );
                }
                _ => String::from("CPU"),
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
                        _ => String::from("-"),
                    },
                    match Command::new("nvidia-smi")
                        .arg("--format=csv,noheader,nounits")
                        .arg("--query-gpu=temperature.gpu")
                        .output()
                    {
                        Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                        _ => String::from("-"),
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
                Ok(battery) => match battery.remaining_capacity {
                    0.9..=1.0 => String::from("󰁹"),
                    0.8..=0.9 => String::from("󰂂"),
                    0.7..=0.8 => String::from("󰂁"),
                    0.6..=0.7 => String::from("󰂀"),
                    0.5..=0.6 => String::from("󰁿"),
                    0.4..=0.5 => String::from("󰁾"),
                    0.3..=0.4 => String::from("󰁽"),
                    0.2..=0.3 => String::from("󰁼"),
                    0.1..=0.2 => String::from("󰁻"),
                    0.0..=0.1 => String::from("󰁺"),
                    _ => String::from("󰂎"),
                },
                _ => String::from("󰂎"),
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
                _ => String::from("?"),
            },
            5,
        ),
        Stat::new(|_| String::from("|"), 0),
        /*Stat::new(
            //VOLUME
            |_| {
                match Command::new("sh")
                            .arg("-c")
                            .arg("amixer sget Master | grep \\[on\\] ")
                            .output(){
                                //| awk -F\"[][]\" '/Left:/ { gsub(\"%\",\"\"); if($4==\"on\"){ if($2 <= 25) print \"󰕿\"; else if($2 <= 75) print \"󰖀\"; else if($2<=100) print \"󰕾\"; } else print \"󰝟\"; }'
                                Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                                _ => String::from("󰝟")
                            }
            },
            -1,
        ),*/
        Stat::new(
            //BRIGHTNESS
            |_| match Command::new("sh")
                .arg("-c")
                .arg("xbacklight -get | sed 's/\\..*//'")
                .output()
            {
                Ok(s) => {
                    match String::from_utf8_lossy(&s.stdout)
                        .trim()
                        .to_string()
                        .parse::<u8>()
                    {
                        Ok(n) => match n {
                            91..=100 => String::from("󰛨"),
                            81..=90 => String::from("󱩖"),
                            71..=80 => String::from("󱩕"),
                            61..=70 => String::from("󱩔"),
                            51..=60 => String::from("󱩓"),
                            41..=50 => String::from("󱩒"),
                            31..=40 => String::from("󱩑"),
                            21..=30 => String::from("󱩐"),
                            11..=20 => String::from("󱩏"),
                            1..=10 => String::from("󱩎"),
                            _ => String::from("󰛩"),
                        },
                        _ => String::from("󰛩"),
                    }
                }
                _ => String::from("󰛩"),
            },
            -2,
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
                    }
                    let s = s.split_whitespace().collect::<Vec<_>>()[2];
                    let num: Result<u8, _> = s[..s.len() - 1].parse();
                    match num {
                        Ok(n) => match n {
                            51.. => String::from("󰤨"),
                            31..=50 => String::from("󰤥"),
                            17..=30 => String::from("󰤢"),
                            1..=16 => String::from("󰤟"),
                            _ => String::from("󰤯"),
                        },
                        _ => String::from("󰤯"),
                    }
                }
                _ => String::from("󰤯"),
            },
            5,
        ),
        Stat::new(
            //BLUETOOTH
            |_| String::from("󰂯"),
            10,
        ),
        Stat::new(|_| String::from("|"), 0),
        Stat::new(
            //KEYBOARD
            |_| match Command::new("sh")
                .arg("-c")
                .arg("xkb-switch | cut -d '(' -f 1")
                .output()
            {
                Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                _ => String::from("--"),
            },
            -3,
        ),
        Stat::new(|_| String::from(" \\"), 0),
        Stat::new(
            //USER@HOST
            |_| {
                format!(
                    "{}@{}",
                    match Command::new("whoami").output() {
                        Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                        _ => String::from("user"),
                    },
                    match Command::new("uname").arg("-n").output() {
                        Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                        _ => String::from("host"),
                    }
                )
            },
            0,
        ),
    ];

    for stat in &mut stats {
        if stat.i <= 0 {
            stat.fetch(&sys);
        }
    }

    let mut i: u8 = 0;

    loop {
        sleep(INTERVAL);
        let sig: i8 = unsafe { S };
        if sig != 0 {
            unsafe { S = 0 };
            let mut s: String = String::new();
            for stat in &mut stats {
                if stat.i == sig {
                    stat.fetch(&sys);
                }
                s += &stat.s;
            }
            setbar(s);
        }
        let mut s: String = String::new();
        for stat in &mut stats {
            if stat.i > 0 && i % stat.i as u8 == 0 {
                stat.fetch(&sys);
            }
            s += &stat.s;
        }
        i = i.wrapping_add(1);
        setbar(s);
    }
}

const INTERVAL: Duration = Duration::from_millis(500);

fn setbar(s: String) {
    let _ = Command::new("xsetroot").arg("-name").arg(s).output();
}
