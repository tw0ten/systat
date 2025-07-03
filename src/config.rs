use crate::{stat::Stat, *};
use chrono::{Datelike, Local};
use std::{fs::File, io::Read, process::Stdio};
use systemstat::ByteSize;

pub fn set(s: &str) {
	_ = Command::new("echo").arg(s).status()
}

fn notify(t: &str, s: &str, e: usize) {
	_ = Command::new("notify-send")
		.arg(t)
		.arg(s)
		.arg("-t")
		.arg(e.to_string())
		.status()
}

pub const MANUAL: [&str; 4] = ["systat-", "volume", "brightness", "keyboard"];
pub fn get() -> [Stat; 23] {
	const ERROR: &str = "#";
	set(ERROR);

	fn utf8(i: &[u8]) -> String {
		String::from_utf8_lossy(i).to_string()
	}

	fn separator() -> Stat {
		Stat::new(|_| String::from(" \\"), 0)
	}

	[
		Stat::new(
			// mount
			|sys| match sys.mount_at("/") {
				Ok(v) => {
					let s = ByteSize::gib(v.avail.as_u64()).to_string();
					format!("({}G)", &s[0..s.len() - 3])
				}
				_ => format!("({})", ERROR),
			},
			30,
		),
		Stat::new(|_| (String::from(" ")), 0),
		Stat::new(
			// ram
			|sys| match sys.memory() {
				Ok(v) => {
					let t = v.total.as_u64();
					format!("<{}", (t - v.free.as_u64()) * 100 / t)
				}
				_ => format!("<{}", ERROR),
			},
			2,
		),
		Stat::new(
			// swap
			|sys| -> String {
				match sys.swap() {
					Ok(swap) => match swap.total.as_u64() {
						0 => String::from("%>"),
						v => format!("+{}%>", (v - swap.free.as_u64()) * 100 / v),
					},
					_ => format!("+{}%>", ERROR),
				}
			},
			4,
		),
		Stat::new(|_| (String::from(" {")), 0),
		Stat::new(
			// cpu usage + temperature
			|sys| -> String {
				match sys.cpu_load_aggregate() {
					Ok(cpu) => {
						thread::sleep(Duration::from_millis(500));
						format!(
							"c={}%{}°",
							match cpu.done() {
								Ok(v) => ((1.0 - v.idle) * 100.0f32).round().to_string(),
								_ => String::from(ERROR),
							},
							match Command::new("cpu-temp").output() {
								Ok(s) => utf8(&s.stdout).to_string(),
								_ => String::from(ERROR),
							},
						)
					}
					_ => String::from("c"),
				}
			},
			2,
		),
		separator(),
		Stat::new(
			// gpu usage + temperature
			|_| {
				format!(
					"g={}%{}°",
					match Command::new("nvidia-smi")
						.arg("--format=csv,noheader,nounits")
						.arg("--query-gpu=utilization.gpu")
						.output()
					{
						Ok(s) => utf8(&s.stdout).trim().to_string(),
						_ => String::from(ERROR),
					},
					match Command::new("nvidia-smi")
						.arg("--format=csv,noheader,nounits")
						.arg("--query-gpu=temperature.gpu")
						.output()
					{
						Ok(s) => utf8(&s.stdout).trim().to_string(),
						_ => String::from(ERROR),
					}
				)
			},
			2,
		),
		Stat::new(|_| (String::from("} ")), 0),
		Stat::new(
			// date & time
			|_| {
				let t = Local::now();
				format!(
					"[{}-{}|{}]",
					t.format("%y/%m/%d"),
					t.weekday().num_days_from_sunday(),
					t.format("%H:%M"),
				)
			},
			1,
		),
        separator(),
		Stat::new(
			// battery
			|sys| match sys.battery_life() {
				Ok(battery) => match &battery.remaining_capacity {
					0.0 => String::from("󰂎"),
					..0.1 => {
						if !sys.on_ac_power().unwrap_or(false) {
							notify(
								"battery",
								format!("{}%", (100.0 * battery.remaining_capacity) as u8).as_str(),
								3000,
							);
						}
						String::from("󰁺")
					}
					..0.2 => String::from("󰁻"),
					..0.3 => String::from("󰁼"),
					..0.4 => String::from("󰁽"),
					..0.5 => String::from("󰁾"),
					..0.6 => String::from("󰁿"),
					..0.7 => String::from("󰂀"),
					..0.8 => String::from("󰂁"),
					..0.9 => String::from("󰂂"),
					_ => String::from("󰁹"),
				},
				_ => String::from(ERROR),
			},
			10,
		),
		Stat::new(
			// ac
			|sys| match sys.on_ac_power() {
				Ok(v) => match v {
					true => String::new(),
					_ => String::from("-"),
				},
				_ => String::from(ERROR),
			},
			5,
		),
		Stat::new(|_| (String::from("|")), 0),
		Stat::new(
			// volume
			|_| match Command::new("pamixer").arg("--get-mute").output() {
				Ok(s) => {
					if utf8(&s.stdout).trim() == "true" {
						return String::from("󰝟");
					}
					match Command::new("pamixer").arg("--get-volume").output() {
						Ok(n) => match utf8(&n.stdout).trim().parse::<u8>() {
							Ok(n) => match n {
								0 => String::from("󰝟"),
								..33 => String::from("󰕿"),
								..66 => String::from("󰖀"),
								_ => String::from("󰕾"),
							},
							_ => String::from(ERROR),
						},
						_ => String::from(ERROR),
					}
				}
				_ => String::from(ERROR),
			},
			-1,
		),
		Stat::new(
			// brightness
			|_| match Command::new("brightnessctl").arg("get").output() {
				Ok(s) => match utf8(&s.stdout).trim().parse::<f64>() {
					Ok(n) => match n as u8 {
						91.. => String::from("󰛨"),
						81.. => String::from("󱩖"),
						71.. => String::from("󱩕"),
						61.. => String::from("󱩔"),
						51.. => String::from("󱩓"),
						41.. => String::from("󱩒"),
						31.. => String::from("󱩑"),
						21.. => String::from("󱩐"),
						11.. => String::from("󱩏"),
						1.. => String::from("󱩎"),
						0 => String::from("󰛩"),
					},
					_ => String::from(ERROR),
				},
				_ => String::from(ERROR),
			},
			-2,
		),
		Stat::new(|_| (String::from("|")), 0),
		Stat::new(
			// wifi
			|_| match File::open("/proc/net/wireless") {
				Ok(mut file) => {
					let mut s = String::new();
					_ = file.read_to_string(&mut s);
					let s = s.split("\n").collect::<Vec<_>>()[2];
					if s.len() < 3 {
						return String::from("󰤯");
					}
					let s = s.split_whitespace().collect::<Vec<_>>()[2];
					match s[..s.len() - 1].parse::<u8>() {
						Ok(n) => match n {
							51.. => String::from("󰤨"),
							31.. => String::from("󰤥"),
							17.. => String::from("󰤢"),
							1.. => String::from("󰤟"),
							0 => String::from("󰤯"),
						},
						_ => String::from(ERROR),
					}
				}
				_ => String::from(ERROR),
			},
			5,
		),
		Stat::new(
			// bluetooth
			|_| match Command::new("systemctl")
				.arg("status")
				.arg("bluetooth")
				.stdout(Stdio::null())
				.status()
			{
				Ok(s) => match s.success() {
					true => String::from("󰂯"),
					_ => String::from("󰂲"),
				},
				_ => String::from(ERROR),
			},
			20,
		),
		Stat::new(|_| (String::from("|")), 0),
		Stat::new(
			// keyboard
			|_| match Command::new("xkb-switch").output() {
				Ok(s) => utf8(&s.stdout).trim().to_string(),
				_ => String::from(ERROR),
			},
			-3,
		),
		separator(),
		Stat::new(
			// user@host
			|_| {
				format!(
					"{}@{}",
					utf8(&Command::new("whoami").output().unwrap().stdout).trim(),
					utf8(&Command::new("uname").arg("-n").output().unwrap().stdout).trim()
				)
			},
			0,
		),
	]
}
