use crate::{stat::Stat, *};
use chrono::{Datelike, Local};
use std::{fs::File, io::Read, process::Stdio};

pub const PREFIX: &str = "systat-";
pub const MANUAL: [&str; 3] = ["volume", "brightness", "keyboard"];

pub fn set(s: &str) {
	_ = Command::new("xsetroot").arg("-name").arg(s).status()
}

const ERROR: &str = "#";
pub fn get() -> [Stat; 23] {
	set(ERROR);
	[
		Stat::new(
			//MOUNT
			|sys| match sys.mount_at("/") {
				Ok(v) => {
					let s = v.avail.to_string();
					format!("(MNT:{}G)", &s[..s.len() - 3])
				}
				_ => String::from("(MNT)"),
			},
			30,
		),
		Stat::new(|_| String::from(" "), 0),
		Stat::new(
			//RAM
			|sys| match sys.memory() {
				Ok(v) => {
					let t = v.total.as_u64();
					format!("<RAM:{}", (t - v.free.as_u64()) * 100 / t)
				}
				_ => format!("<RAM:{}", ERROR),
			},
			2,
		),
		Stat::new(
			//SWAP
			|sys| -> String {
				match sys.swap() {
					Ok(swap) => match swap.total.as_u64() {
						0 => String::from("%>"),
						v => format!(":{}%>", (v - swap.free.as_u64()) * 100 / v),
					},
					_ => format!(":{}%>", ERROR),
				}
			},
			4,
		),
		Stat::new(|_| String::from(" {"), 0),
		Stat::new(
			//CPU USAGE + TEMPERATURE
			|sys| -> String {
				match sys.cpu_load_aggregate() {
					Ok(cpu) => {
						thread::sleep(Duration::from_millis(500));
						format!(
							"CPU:{}%{}°",
							match cpu.done() {
								Ok(v) => ((1.0 - v.idle) * 100.0f32).round().to_string(),
								_ => String::from(ERROR),
							},
							match Command::new("cpu-temp").output() {
								Ok(s) => String::from_utf8_lossy(&s.stdout).to_string(),
								_ => String::from(ERROR),
							},
						)
					}
					_ => String::from("CPU"),
				}
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
						_ => String::from(ERROR),
					},
					match Command::new("nvidia-smi")
						.arg("--format=csv,noheader,nounits")
						.arg("--query-gpu=temperature.gpu")
						.output()
					{
						Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
						_ => String::from(ERROR),
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
				format!(
					"[{}.{}]",
					t.weekday().num_days_from_sunday(),
					t.format("%d/%m|%H:%M")
				)
			},
			1,
		),
		Stat::new(|_| String::from(" \\"), 0),
		Stat::new(
			//BATTERY
			|sys| match sys.battery_life() {
				Ok(battery) => match battery.remaining_capacity {
					0.0 => String::from("󰂎"),
					..0.1 => String::from("󰁺"),
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
			//AC
			|sys| match sys.on_ac_power() {
				Ok(v) => match v {
					true => String::new(),
					_ => String::from("-"),
				},
				_ => String::from(ERROR),
			},
			5,
		),
		Stat::new(|_| String::from("|"), 0),
		Stat::new(
			//VOLUME
			|_| match Command::new("pamixer").arg("--get-mute").output() {
				Ok(s) => {
					if String::from_utf8_lossy(&s.stdout).trim() == "true" {
						return String::from("󰝟");
					}
					match Command::new("pamixer").arg("--get-volume").output() {
						Ok(n) => match String::from_utf8_lossy(&n.stdout).trim().parse::<u8>() {
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
			//BRIGHTNESS
			|_| match Command::new("xbacklight").arg("-get").output() {
				Ok(s) => match String::from_utf8_lossy(&s.stdout).trim().parse::<f64>() {
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
		Stat::new(|_| String::from("|"), 0),
		Stat::new(
			//WIFI
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
			//BLUETOOTH
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
		Stat::new(|_| String::from("|"), 0),
		Stat::new(
			//KEYBOARD
			|_| match Command::new("xkb-switch").output() {
				Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
				_ => String::from(ERROR),
			},
			-3,
		),
		Stat::new(|_| String::from(" \\"), 0),
		Stat::new(
			//USER@HOST
			|_| {
				format!(
					"{}@{}",
					String::from_utf8_lossy(&Command::new("whoami").output().unwrap().stdout)
						.trim(),
					String::from_utf8_lossy(
						&Command::new("uname").arg("-n").output().unwrap().stdout
					)
					.trim()
				)
			},
			0,
		),
	]
}
