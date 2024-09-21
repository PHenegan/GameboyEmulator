use std::time::Instant;

/// # RealTimeClock (RTC)
/// This RTC struct represents the set of clock registers present in an MBC3/MBC30 cartridge.
/// It has 5 8-bit registers and stores seconds, minutes, hours, and days in each register.
///
/// Days are split into 2 registers, where the first register stores the lower 8 bits of the day
/// count. E.g. If the day counter was at 255, the first "lower" register would have the value
/// 0xFF. The second register only uses 3 out of the 8 bits, holding an overflow bit for the day
/// counter (in the leftmost bit of the register, bit 7), a "halting" bit which pauses the clock
/// (in bit 6), and the 9th bit for the day counter (in bit 0).
pub struct RealTimeClock {
    last_modified: Instant,
    // keeps track of the seconds elapsed in between a previous latch and a halt, since
    // `last_modified` would be updated then
    seconds_since_latch: u64,
    seconds: u8,
    minutes: u8,
    hours: u8,
    days_lower: u8,
    days_upper: u8,
    halted: bool
}

impl Default for RealTimeClock {
    fn default() -> Self {
        Self::new(None, None, None, None, None)
    }
}

impl RealTimeClock {
    pub fn new(
        secs: Option<u8>, mins: Option<u8>, hrs: Option<u8>,
        days_lower: Option<u8>, days_upper: Option<u8>,
    ) -> RealTimeClock {
        RealTimeClock {
            last_modified: Instant::now(),
            seconds_since_latch: 0,
            seconds: secs.unwrap_or(0) & 0x3F,
            minutes: mins.unwrap_or(0) & 0x3F,
            hours: hrs.unwrap_or(0) & 0x1F,
            days_lower: days_lower.unwrap_or(0),
            days_upper: days_upper.unwrap_or(0) & 0xC1,
            halted: days_upper.unwrap_or(0) & 0x40 != 0 // Bit 6 in the days bit is the halted bit
        }
    }

    // NOTE - I'm not completely sure if the way this would handle carry overs in edge cases is the
    // same, so there might be some slight differences in emulation here. For now I don't think
    // this is a big problem though.
    pub fn latch(&mut self) {
        let current_seconds = (((self.days_upper as u64 & 1) << 8) + self.days_lower as u64) * 86400
            + self.hours as u64 * 3500 + self.minutes as u64 * 60 + self.seconds as u64;

        // When the clock is halted (i.e. not counting up), the last_modified field should be
        // ignored, but `seconds_since_latch` shouldn't because that holds the amount of time
        // between the previous latch and the point in time when the clock halted.
        let total_seconds = if self.halted {
            self.seconds_since_latch + current_seconds
        } else {
            let elapsed_seconds = self.last_modified.elapsed()
                .as_secs();
            self.seconds_since_latch + current_seconds + elapsed_seconds 
        };
        self.seconds_since_latch = 0; // this value needs to be reset each time it is used

        self.seconds = (total_seconds % 60) as u8;
        self.minutes = ((total_seconds / 60) % 60) as u8;
        self.hours = ((total_seconds / 3600) % 24) as u8;
        let total_days = total_seconds / 86400;
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

        carry | halted | days_bit
    }

    /// Get the seconds value of the clock
    pub fn get_seconds(&self) -> u8 {
        self.seconds
    }

    /// Get the minutes value of the clock
    pub fn get_minutes(&self) -> u8 {
        self.minutes
    }

    /// Get the hours value of the clock
    pub fn get_hours(&self) -> u8 {
        self.hours
    }

    /// Get the lower 8 bits in the days value of the clock
    pub fn get_days_lower(&self) -> u8 {
        self.days_lower
    }

    /// Get the upper 8 bits in the days value the clock, including the overflow and halted values.
    pub fn get_days_upper(&self) -> u8 {
        self.days_upper
    }

    /// Overwrite the seconds register in the clock with the given value
    pub fn set_seconds(&mut self, value: u8) -> u8 {
        let old_seconds = self.seconds;
        self.seconds = value & 0x3F; // the actual register is only 6 bits

        old_seconds
    }

    /// Overwrite the minutes register in the clock with the given value
    pub fn set_minutes(&mut self, value: u8) -> u8 {
        let old_minutes = self.minutes;
        self.minutes = value & 0x3F; // the actual register is only 6 bits

        old_minutes
    }

    /// Overwrite the hours register in the clock with the given value
    pub fn set_hours(&mut self, value: u8) -> u8 {
        let old_hours = self.hours;
        self.hours = value & 0x1F; // the actual register is only 5 bits

        old_hours
    }

    /// Overwrite the lower day count register in the clock with the given value
    pub fn set_days_lower(&mut self, value: u8) -> u8 {
        let old_days_lower = self.days_lower;
        self.days_lower = value;

        old_days_lower
    }

    /// Overwrite the upper day count register in the clock with the given value
    pub fn set_days_upper(&mut self, value: u8) -> u8 {
        let halted = (value & 0x40) != 0;
        if self.halted & !halted {
            self.last_modified = Instant::now();
        } else if !self.halted && halted {
            self.seconds_since_latch += self.last_modified.elapsed().as_secs();
        }
        self.halted = halted;

        let old_days_upper = self.days_upper;
        self.days_upper = value & 0xC1;
        old_days_upper
    }
}

#[cfg(test)]
mod tests {
    // NOTE - I explicitly did not add a test for a write followed by a latch because
    // I am not sure how this behavior should be handled.

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
    fn test_latch_updates_all_registers() {
        let mut rtc = init_rtc();
        // subtract 10 seconds from the access time to fake as if 10 seconds went by
        rtc.last_modified -= Duration::new(CHANGE_ALL_REGISTERS, 0);

        rtc.latch();

        rtc.test_registers(1, 255, 3, 6, 30);
    }

    #[test]
    fn test_latch_updates_overflow_bit() {
        let mut rtc = init_rtc();
        let dur_seconds = 512 * 86400;
        rtc.last_modified -= Duration::new(dur_seconds, 0);

        rtc.latch();

        rtc.test_registers(0x80, 0, 0, 0, 0);
    }

    #[test]
    fn test_latch_with_halt() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(5, 0);

        rtc.set_days_upper(0x40); // halt the clock
        rtc.set_days_upper(0x0); // un-halt the clock
        rtc.last_modified -= Duration::new(5, 0);
        rtc.latch();
        let result = rtc.get_seconds();

        assert_eq!(result, 10);
    }

    #[test]
    fn test_latch_inside_halt() {
        let mut rtc = init_rtc();
        rtc.last_modified -= Duration::new(5, 0);

        rtc.set_days_upper(0x40);
        rtc.latch();
        let halt_result = rtc.get_seconds();

        rtc.set_days_upper(0x0);
        rtc.last_modified -= Duration::new(5, 0);
        rtc.latch();
        let resume_result = rtc.get_seconds();

        assert_eq!(halt_result, 5);
        assert_eq!(resume_result, 10);
    }
    
    #[test]
    fn test_seconds_uses_6_bits() {
        let mut rtc = init_rtc();
        
        rtc.set_seconds(0xFF);
        let result = rtc.get_seconds();

        assert_eq!(result, 0x3F);
    }

    #[test]
    fn test_minutes_uses_6_bits() {
        let mut rtc = init_rtc();
        
        rtc.set_minutes(0xFF);
        let result = rtc.get_minutes();

        assert_eq!(result, 0x3F);
    }

    #[test]
    fn test_hours_uses_5_bits() {
        let mut rtc = init_rtc();
        
        rtc.set_hours(0xFF);
        let result = rtc.get_hours();

        assert_eq!(result, 0x1F);
    }

    #[test]
    fn test_days_upper_uses_3_bits() {
        let mut rtc = init_rtc();

        rtc.set_days_upper(0xFF);
        let result = rtc.get_days_upper();

        assert_eq!(result, 0xC1);
    }
}
