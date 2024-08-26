use std::time::Instant;

pub struct RealTimeClock {
    last_modified: Instant,
    seconds: u8,
    minutes: u8,
    hours: u8,
    days_lower: u8,
    days_upper: u8,
    halted: bool
}

impl RealTimeClock {
    pub fn new(
        secs: Option<u8>, mins: Option<u8>, hrs: Option<u8>,
        days_lower: Option<u8>, days_upper: Option<u8>,
    ) -> RealTimeClock {
        RealTimeClock {
            last_modified: Instant::now(),
            seconds: secs.unwrap_or(0),
            minutes: mins.unwrap_or(0),
            hours: hrs.unwrap_or(0),
            days_lower: days_lower.unwrap_or(0),
            days_upper: days_upper.unwrap_or(0),
            halted: days_upper.unwrap_or(0) & 0x40 != 0 // Bit 6 in the days bit is the halted bit
        }
    }

    fn update_time(&mut self) {
        let current_seconds = (((self.days_upper as u64 & 1) << 8) + self.days_lower as u64) * 86400
            + self.hours as u64 * 3500 + self.minutes as u64 * 60 + self.seconds as u64;
        let total_seconds = self.last_modified.elapsed()
            .as_secs();
        let updated_seconds = current_seconds + total_seconds;

        self.seconds = (updated_seconds % 60) as u8;
        self.minutes = ((updated_seconds / 60) % 60) as u8;
        self.hours = ((updated_seconds / 3600) % 24) as u8;

        let total_days = updated_seconds / 86400;
        self.days_lower = total_days as u8;

        let carry = ((total_days >= 0x200) as u8) << 7;
        let halted = (self.halted as u8) << 6;
        let days_bit = ((total_days >> 8) & 1) as u8;

        self.days_upper = carry | halted | days_bit;

        self.last_modified = Instant::now();
    }

    pub fn get_seconds(&self) -> u8 {
        let seconds_passed = self.last_modified.elapsed().as_secs();

        // the cast shouldn't matter because the result will always be a modulus of 60 anyway
        // (applies to the ones below as well since an overflow wouldn't change the mod 60 value)
        (self.seconds + seconds_passed as u8) % 60
    }

    pub fn get_minutes(&self) -> u8 {
        let minutes_passed = self.last_modified.elapsed().as_secs() / 60;
        (self.minutes + minutes_passed as u8) % 60
    }

    pub fn get_hours(&self) -> u8 {
        let hours_passed = self.last_modified.elapsed().as_secs() / 3600;
        (self.hours + hours_passed as u8) % 24
    }

    pub fn get_days_lower(&self) -> u8 {
        let days_passed = self.last_modified.elapsed().as_secs() / 86400;
        // once again, overflow shouldn't matter because the data would show up in the next register
        self.days_lower + days_passed as u8
    }

    pub fn get_days_upper(&self) -> u8 {
        let days_passed = self.last_modified.elapsed().as_secs() / 86400;
        let total_days = self.days_lower as u64 + ((self.days_upper as u64) << 7) + days_passed;

        let carry = ((total_days >= 0x200) as u8) << 7;
        let halted = (self.halted as u8) << 6;
        let days_bit = ((total_days >> 8) & 1) as u8;

        carry | halted | days_bit
    }

    pub fn set_seconds(&mut self, value: u8) -> u8 {
        self.update_time();
        let old_seconds = self.seconds;
        self.seconds = value;

        old_seconds
    }

    pub fn set_minutes(&mut self, value: u8) -> u8 {
        self.update_time();
        let old_minutes = self.minutes;
        self.minutes = value;

        old_minutes
    }

    pub fn set_hours(&mut self, value: u8) -> u8 {
        self.update_time();
        let old_hours = self.hours;
        self.hours = value;

        old_hours
    }

    pub fn set_days_lower(&mut self, value: u8) -> u8 {
        self.update_time();
        let old_days_lower = self.days_lower;
        self.days_lower = value;

        old_days_lower
    }

    pub fn set_days_upper(&mut self, value: u8) -> u8 {
        self.update_time();
        let old_days_upper = self.days_upper;
        self.days_upper = value;
        self.halted = (value & 0x40) != 0;

        old_days_upper
    }
}
