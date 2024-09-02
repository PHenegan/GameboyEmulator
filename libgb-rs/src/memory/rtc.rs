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
// TODO - If a call to set seconds, minutes, or hours is greater than the proper range (60/24),
// should it impact the next unit of time?
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
        self.days_upper = self.create_days_upper(total_days);

        self.last_modified = Instant::now();
    }

    fn create_days_upper(&self, total_days: u64) -> u8 {
        // NOTE - the carry flag should never be "unset" unless explicitly done so by the
        // program
        let carry = (((total_days >= 0x200) as u8) << 7) | (self.days_upper & 0x80);
        let halted = (self.halted as u8) << 6;
        let days_bit = ((total_days >> 8) & 1) as u8;
        let middle_bits = self.days_upper & 0x3E; // preserve the middle 5 bits

        // TODO - is there defined behavior for how the middle 5 bits are set? Should I be trying
        // to preserve them like the carry bit?
        carry | halted | middle_bits | days_bit
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
        let total_days = self.days_lower as u64 + ((self.days_upper as u64 & 1) << 8) + days_passed;

        self.create_days_upper(total_days)
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
        let halted = (value & 0x40) != 0;
        self.days_upper = value;

        if self.halted & !halted {
            self.last_modified = Instant::now();
        }

        self.halted = halted;

        old_days_upper
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    const CHANGE_ALL_REGISTERS: u64 = 86400 * 511 + 11190;

    fn init_rtc() -> RealTimeClock {
        RealTimeClock::new(None, None, None, None, None)
    }

    impl RealTimeClock {
        fn test_registers(&self, days_up: u8, days_low: u8, hrs: u8, mins: u8, secs: u8) {
            let seconds = self.get_seconds();
            let minutes = self.get_minutes();
            let hours = self.get_hours();
            let days_lower = self.get_days_lower();
            let days_upper = self.get_days_upper();

            assert_eq!(seconds, secs, "seconds should be updated correctly");
            assert_eq!(minutes, mins, "minutes should be updated correctly");
            assert_eq!(hours, hrs, "hours should be updated correctly");
            assert_eq!(days_lower, days_low, "days (lower register) should be updated correctly");
            assert_eq!(days_upper, days_up, "days (upper register) should be updated correctly");
        }
    }

    #[test]
    fn test_updates_seconds() {
        let mut rtc = init_rtc();
        // subtract 10 seconds from the access time to fake as if 10 seconds went by
        rtc.last_modified -= Duration::new(10, 0);

        rtc.test_registers(0, 0, 0, 0, 10);
    }

    #[test]
    fn test_updates_minutes() {
        let mut rtc = init_rtc();
        
        rtc.last_modified -= Duration::new(90, 0);

        rtc.test_registers(0, 0, 0, 1, 30);
    }

    #[test]
    fn test_updates_hours() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(7321, 0);

        rtc.test_registers(0, 0, 2, 2, 1);
    }

    #[test]
    fn test_updates_days_lower() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(270_183, 0);

        rtc.test_registers(0, 3, 3, 3, 3);
    }

    #[test]
    fn test_updates_days_upper() {
        let mut rtc = init_rtc();
        let dur_seconds = 511 * 86400 + 3842;
        rtc.last_modified -= Duration::new(dur_seconds, 0);

        rtc.test_registers(1, 255, 1, 4, 2);
    }

    #[test]
    fn test_updates_overflow_bit() {
        let mut rtc = init_rtc();
        let dur_seconds = 512 * 86400;
        rtc.last_modified -= Duration::new(dur_seconds, 0);

        rtc.test_registers(0x80, 0, 0, 0, 0);
    }

    #[test]
    fn test_set_seconds() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(61, 0);

        rtc.set_seconds(5);

        rtc.test_registers(0, 0, 0, 1, 5);
    }

    #[test]
    fn test_set_minutes() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(3661, 0);

        rtc.set_minutes(42);

        rtc.test_registers(0, 0, 1, 42, 1);
    }

    #[test]
    fn test_set_hours() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(86625, 0);

        rtc.set_hours(2);

        rtc.test_registers(0, 1, 2, 3, 45);
    }
    
    #[test]
    fn test_set_days_lower() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(511 * 86400 + 7420, 0);

        rtc.set_days_lower(42);

        rtc.test_registers(1, 42, 2, 3, 40)
    }

    #[test]
    fn test_set_days_upper() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(30 * 86400 + 11190, 0);

        rtc.set_days_upper(0xFF);

        rtc.test_registers(0xFF, 30, 3, 6, 30);
    }

    #[test]
    fn test_halted_stops_getter_update() {
        let mut rtc = init_rtc();
        
        rtc.set_days_upper(0x40);
        rtc.last_modified -= Duration::new(CHANGE_ALL_REGISTERS, 0);

        rtc.test_registers(0x40, 0, 0, 0, 0);
    }

    #[test]
    fn test_halted_stops_set_seconds_update() {
        let mut rtc = init_rtc();

        rtc.set_days_upper(0x40);
        rtc.last_modified -= Duration::new(CHANGE_ALL_REGISTERS, 0);
        rtc.set_seconds(5);

        rtc.test_registers(0x40, 0, 0, 0, 5);
    }

    #[test]
    fn test_halted_stops_minutes_update() {
        let mut rtc = init_rtc();

        rtc.set_days_upper(0x40);
        rtc.last_modified -= Duration::new(CHANGE_ALL_REGISTERS, 0);
        rtc.set_minutes(5);

        rtc.test_registers(0x40, 0, 0, 5, 0);
    }

    #[test]
    fn test_halted_stops_hours_update() {
        let mut rtc = init_rtc();

        rtc.set_days_upper(0x40);
        rtc.last_modified -= Duration::new(CHANGE_ALL_REGISTERS, 0);
        rtc.set_hours(5);

        rtc.test_registers(0x40, 0, 5, 0, 0);
    }

    #[test]
    fn test_halted_stops_days_lower_update() {
        let mut rtc = init_rtc();

        rtc.set_days_upper(0x40);
        rtc.last_modified -= Duration::new(CHANGE_ALL_REGISTERS, 0);
        rtc.set_days_lower(5);

        rtc.test_registers(0x40, 5, 0, 0, 0);
    }

    #[test]
    fn test_halted_stops_days_upper_update() {
        let mut rtc = init_rtc();

        rtc.set_days_upper(0x40);
        rtc.last_modified -= Duration::new(CHANGE_ALL_REGISTERS, 0);
        rtc.set_days_upper(0xBF);

        rtc.test_registers(0xBF, 0, 0, 0, 0);
    }
}
