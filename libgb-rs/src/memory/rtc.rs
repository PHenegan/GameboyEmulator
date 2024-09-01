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
        if self.halted {
            return;
        }

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
        if self.halted {
            return self.seconds;
        }

        let seconds_passed = self.last_modified.elapsed().as_secs();
        let updated_seconds = self.seconds as u64 + seconds_passed;
        (updated_seconds % 60) as u8
    }

    pub fn get_minutes(&self) -> u8 {
        if self.halted {
            return self.minutes;
        }

        let minutes_passed = self.last_modified.elapsed().as_secs() / 60;
        let updated_minutes = self.minutes as u64 + minutes_passed;
        (updated_minutes % 60) as u8
    }

    pub fn get_hours(&self) -> u8 {
        if self.halted {
            return self.hours;
        }

        let hours_passed = self.last_modified.elapsed().as_secs() / 3600;
        let updated_hours = self.hours as u64 + hours_passed;
        (updated_hours % 24) as u8
    }

    pub fn get_days_lower(&self) -> u8 {
        if self.halted {
            return self.days_lower;
        }

        let days_passed = self.last_modified.elapsed().as_secs() / 86400;
        // Overflow shouldn't matter because the data would show up in the next register
        self.days_lower + days_passed as u8
    }

    pub fn get_days_upper(&self) -> u8 {
        if self.halted {
            return self.days_upper;
        }
            
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn init_rtc() -> RealTimeClock {
        RealTimeClock::new(None, None, None, None, None)
    }

    fn test_registers(rtc: &RealTimeClock, days_up: u8, days_low: u8, hrs: u8, mins: u8, secs: u8) {
        let seconds = rtc.get_seconds();
        let minutes = rtc.get_minutes();
        let hours = rtc.get_hours();
        let days_lower = rtc.get_days_lower();
        let days_upper = rtc.get_days_upper();

        assert_eq!(seconds, secs, "seconds should be updated correctly");
        assert_eq!(minutes, mins, "minutes should be updated correctly");
        assert_eq!(hours, hrs, "hours should be updated correctly");
        assert_eq!(days_lower, days_low, "days (lower register) should be updated correctly");
        assert_eq!(days_upper, days_up, "days (upper register) should be updated correctly");
    }
    
    #[test]
    fn test_updates_seconds() {
        let mut rtc = init_rtc();
        // subtract 10 seconds from the access time to fake as if 10 seconds went by
        rtc.last_modified -= Duration::new(10, 0);

        test_registers(&rtc, 0, 0, 0, 0, 10);
    }

    #[test]
    fn test_updates_minutes() {
        let mut rtc = init_rtc();

        rtc.last_modified -= Duration::new(90, 0);

        test_registers(&rtc, 0, 0, 0, 1, 30);
    }

    #[test]
    fn test_updates_hours() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(7321, 0);

        test_registers(&rtc, 0, 0, 2, 2, 1);
    }

    #[test]
    fn test_updates_days_lower() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(270_183, 0);

        test_registers(&rtc, 0, 3, 3, 3, 3);
    }

    #[test]
    fn test_updates_days_upper() {
        let mut rtc = init_rtc();
        let dur_seconds = 511 * 86400 + 3842;
        rtc.last_modified -= Duration::new(dur_seconds, 0);

        test_registers(&rtc, 1, 255, 1, 4, 2);
    }

    #[test]
    fn test_updates_overflow_bit() {
        let mut rtc = init_rtc();
        let dur_seconds = 512 * 86400;
        rtc.last_modified -= Duration::new(dur_seconds, 0);

        test_registers(&rtc, 0x80, 0, 0, 0, 0);

    }
}
