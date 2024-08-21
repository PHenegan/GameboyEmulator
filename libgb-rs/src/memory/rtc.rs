use std::cell::RefCell;
use std::ops::{AddAssign};
use std::time::{Duration, Instant};

pub struct RealTimeClock {
    last_accessed: RefCell<Instant>,
    time_register: RefCell<Duration>,
    pub halted: bool,
}

impl RealTimeClock {
    fn new(time_register: Option<Duration>, last_accessed: Option<Instant>) -> RealTimeClock {
        let mut time_register = time_register
            .unwrap_or(Duration::new(0, 0));
        if let Some(last_accessed) = last_accessed {
            time_register.add_assign(last_accessed.elapsed());
        }

        RealTimeClock {
            last_accessed: RefCell::new(Instant::now()),
            time_register: RefCell::new(time_register),
            halted: false
        }
    }
    fn update_time(&self) {
        if self.halted {
            return;
        }

        let elapsed = self.last_accessed.borrow()
            .elapsed();
        self.time_register.borrow_mut()
            .add_assign(elapsed);
        self.last_accessed.replace(Instant::now());
    }

    fn split_values(&self) -> (u8, u8, u8, u16) {
        let total_seconds = self.time_register.borrow()
            .as_secs();
        (
            (total_seconds % 60) as u8,
            ((total_seconds / 60) % 60) as u8,
            ((total_seconds / 3600) % 24) as u8,
            ((total_seconds / 86400) & 0x1FF) as u16
        )
    }

    fn set_time(& mut self, seconds: u8, minutes: u8, hours: u8, days_8: u8, days_upper: u8) {
        let total_seconds: u32 = seconds + (minutes * 60) + (hours * 3600) + (days_8 * )
    }

    pub fn get_seconds(&self) -> u8 {
        self.update_time();
        let seconds = self.time_register.borrow().as_secs() % 60;

        seconds as u8
    }

    pub fn get_minutes(&self) -> u8 {
        self.update_time();
        let minutes = (self.time_register.borrow().as_secs() / 60) % 60;

        minutes as u8
    }

    pub fn get_hours(&self) -> u8 {
        self.update_time();
        let hours = (self.time_register.borrow().as_secs() / 3600) % 24;

        hours as u8
    }

    pub fn get_days(&self) -> u16 {
        self.update_time();
        let days = self.time_register.borrow().as_secs() / 86400;

        (days & 0x1FF) as u16
    }

    pub fn set_seconds(&mut self, value: u8) -> u8 {
        self.update_time();
        let (mut seconds, mut minutes, mut hours, mut days) = self.split_values();

        let old_seconds = seconds;

        seconds = value % 60;
        minutes += value / 60;
        hours += minutes / 60;
        days += (hours / 24) as u16;

        minutes = minutes / 60;
        hours = hours / 24;

        old_seconds
    }

    pub fn set_minutes(&self, value: u8) -> {
        self.update_time();
        let (mut _seconds, mut minutes, mut hours, mut days) = self.split_values();

        let minutes

    }
}