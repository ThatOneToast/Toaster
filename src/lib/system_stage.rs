
#[derive(Debug, Clone)]
pub struct Time {
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Time {
    pub fn new(month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Schedule(Time);

impl Schedule {
    pub fn new(time: Time) -> Self {
        Self(time)
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        // format: MM::DD::HH::MM::SS
        let mut time = Time::new(0, 0, 0, 0, 0);
        let mut split = s.split(':');

        if let Some(month) = split.next() {
            time.month = month.parse::<u8>().unwrap();
        } else {
            return Err("Invalid schedule format".to_string());
        }

        if let Some(day) = split.next() {
            time.day = day.parse::<u8>().unwrap();
        } else {
            return Err("Invalid schedule format".to_string());
        }

        if let Some(hour) = split.next() {
            time.hour = hour.parse::<u8>().unwrap();
        } else {
            return Err("Invalid schedule format".to_string());
        }

        if let Some(minute) = split.next() {
            time.minute = minute.parse::<u8>().unwrap();
        } else {
            return Err("Invalid schedule format".to_string());
        }

        if let Some(second) = split.next() {
            time.second = second.parse::<u8>().unwrap();
        } else {
            return Err("Invalid schedule format".to_string());
        }

        Ok(Self(time))
    }

    pub fn get_as_u64(&self) -> u64 {
        let mut time = 0;
        time += self.0.month as u64 * 60 * 60 * 24 * 30;
        time += self.0.day as u64 * 60 * 60 * 24;
        time += self.0.hour as u64 * 60 * 60;
        time += self.0.minute as u64 * 60;
        time += self.0.second as u64;
        time
    }

    /// Checks if the schedule is only for seconds
    /// MM::DD::HH::MM are all 0's if this case is true
    pub fn is_sec_only(&self) -> bool {
        if self.0.month == 0 && self.0.day == 0 && self.0.hour == 0 && self.0.minute == 0 {
            true
        } else {
            false
        }
    }

    /// Checks if the schedule is only for minutes
    /// MM::DD::HH::MM are all 0's if this case is true
    pub fn is_min_only(&self) -> bool {
        if self.0.month == 0 && self.0.day == 0 && self.0.hour == 0 {
            true
        } else {
            false
        }
    }

    /// Checks if the schedule is only for hours
    /// MM::DD::HH::MM are all 0's if this case is true
    pub fn is_hour_only(&self) -> bool {
        if self.0.month == 0 && self.0.day == 0 {
            true
        } else {
            false
        }
    }

    /// Checks if the schedule is only for days
    /// MM::DD::HH::MM are all 0's if this case is true
    pub fn is_day_only(&self) -> bool {
        if self.0.month == 0 {
            true
        } else {
            false
        }
    }

    /// Checks if the schedule is only for months
    /// MM::DD::HH::MM are all 0's if this case is true
    pub fn is_month_only(&self) -> bool {
        if self.0.day == 0 {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct SStage {
    /// The command to be ran on the schedule
    pub command: String,
    /// When does this system run?
    pub schedule: Schedule,
}

impl SStage {
    pub fn new(command: String, schedule: Schedule) -> Self {
        Self { command, schedule }
    }
}