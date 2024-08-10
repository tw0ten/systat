use std::process::{Child, Command};
use std::thread;
use systemstat::System;

use crate::S;

pub struct Stat {
    pub s: String,
    f: fn(&System) -> String,
    pub i: i8,
}

fn spawn_process(i: i8) -> Child {
    Command::new("phandle")
        .arg(format!("systat-subprocess{}", i))
        .spawn()
        .unwrap()
}

impl Stat {
    pub fn new(f: fn(&System) -> String, i: i8) -> Self {
        let st = Stat {
            s: String::new(),
            f,
            i,
        };
        if i < 0 {
            thread::spawn(move || {
                let mut child;
                loop {
                    child = spawn_process(i);
                    child.wait().unwrap();
                    unsafe { S = i };
                }
            });
        }
        st
    }
    pub fn fetch(&mut self, value: &System) {
        self.s = (self.f)(value);
    }
}
