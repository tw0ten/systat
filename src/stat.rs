use nix::sys::wait::waitpid;
use std::process::{Child, Command};
use std::thread;
use systemstat::System;

use crate::S;

pub struct Stat {
    pub s: String,
    f: fn(&System) -> String,
    pub i: i8,
}

const SUBPROCESS_NAME: &str = "systat-subprocess";

impl Stat {
    fn spawn_process(i: i8) -> Child {
        Command::new("dummy")
            .arg(format!("{}{}", SUBPROCESS_NAME, i))
            .spawn()
            .expect(&format!("couldnt start {}{}", SUBPROCESS_NAME, i))
    }

    pub fn new(f: fn(&System) -> String, i: i8) -> Self {
        let st: Stat = Stat {
            s: String::new(),
            f,
            i,
        };
        if i < 0 {
            thread::spawn(move || {
                let mut child: Child = Self::spawn_process(i);
                loop {
                    match waitpid(Some(nix::unistd::Pid::from_raw(child.id() as i32)), None) {
                        Ok(_) => {
                            unsafe { S = i };
                            child = Self::spawn_process(i);
                        }
                        Err(_) => {}
                    }
                }
            });
        }
        return st;
    }

    pub fn fetch(&mut self, value: &System) {
        self.s = (self.f)(value);
    }
}
