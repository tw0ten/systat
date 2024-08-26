use crate::{thread, Command, System, S};

pub struct Stat {
    pub i: i16,
    f: fn(&System) -> String,
    pub s: String,
}

impl Stat {
    pub fn new(f: fn(&System) -> String, i: i16) -> Self {
        if i < 0 {
            thread::spawn(move || loop {
                _ = Command::new("phandle")
                    .arg(format!("systat{}", i))
                    .spawn()
                    .unwrap()
                    .wait();
                unsafe { S = i }
            });
        }
        Self {
            s: String::new(),
            f,
            i,
        }
    }

    pub fn f(&mut self, sys: &System) {
        self.s = (self.f)(sys)
    }
}
