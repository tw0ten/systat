use systemstat::System;

pub struct Stat {
    pub s: String,
    f: fn(&System) -> String,
    pub i: u8
}

impl Stat {
    pub fn new(f: fn(&System) -> String, i: u8) -> Self {
        Stat { s: String::new(), f, i }
    }

    pub fn fetch(&mut self, value: &System) {
        self.s = (self.f)(value);
    }
}
