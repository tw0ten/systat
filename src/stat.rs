use std::{
    process::{Child, Command},
    thread,
};
use systemstat::System;

use crate::S;

pub struct Stat {
    pub s: String,
    f: fn(&System) -> String,
    pub i: i16,
}

fn spawn(i: i16) -> Child {
    Command::new("phandle")
        .arg(format!("systat{}", i))
        .spawn()
        .unwrap()
}

impl Stat {
    pub fn new(f: fn(&System) -> String, i: i16) -> Self {
        let st = Stat {
            s: String::new(),
            f,
            i,
        };
        if i < 0 {
            thread::spawn(move || {
                let mut child;
                loop {
                    child = spawn(i);
                    let _ = child.wait();
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
