pub const TICKS_PER_SIM_SECOND: u32 = 3;
pub const TICKS_PER_SIM_MINUTE: u32 = 60 * TICKS_PER_SIM_SECOND;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticks(pub u32);

impl From<Duration> for Ticks {
    fn from(d_secs: Duration) -> Ticks {
        Ticks(d_secs.0 * TICKS_PER_SIM_SECOND)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Duration(pub u32);

impl Duration {
    pub fn from_seconds(seconds: usize) -> Self {
        Duration(seconds as u32)
    }

    pub fn from_minutes(minutes: usize) -> Self {
        Self::from_seconds(60 * minutes)
    }

    pub fn from_hours(hours: usize) -> Self {
        Self::from_minutes(60 * hours)
    }

    pub fn as_seconds(self) -> f32 {
        self.0 as f32
    }

    pub fn as_minutes(self) -> f32 {
        self.0 as f32 / 60.0
    }

    pub fn as_hours(self) -> f32 {
        self.as_minutes() / 60.0
    }

    pub fn as_days(self) -> f32 {
        self.as_hours() / 24.0
    }
}

impl ::std::ops::Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Duration(self.0 + rhs.0)
    }
}

impl ::std::ops::AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Instant(u32);

impl Instant {
    pub fn new(ticks: usize) -> Self {
        Instant(ticks as u32)
    }

    pub fn ticks(self) -> usize {
        self.0 as usize
    }

    pub fn iticks(self) -> isize {
        self.0 as isize
    }
}

impl<D: Into<Ticks>> ::std::ops::Add<D> for Instant {
    type Output = Self;

    fn add(self, rhs: D) -> Self {
        Instant(self.0 + rhs.into().0)
    }
}

impl<D: Into<Ticks>> ::std::ops::AddAssign<D> for Instant {
    fn add_assign(&mut self, rhs: D) {
        self.0 += rhs.into().0
    }
}

impl<D: Into<Ticks>> ::std::ops::Sub<D> for Instant {
    type Output = Self;

    fn sub(self, rhs: D) -> Self {
        Instant(self.0 - rhs.into().0)
    }
}

impl<D: Into<Ticks>> ::std::ops::SubAssign<D> for Instant {
    fn sub_assign(&mut self, rhs: D) {
        self.0 -= rhs.into().0
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TimeOfDay {
    minutes_of_day: u16,
}

const BEGINNING_TIME_OF_DAY: usize = 7;
const MINUTES_PER_DAY: usize = 60 * 24;

impl TimeOfDay {
    pub fn new(h: usize, m: usize) -> Self {
        TimeOfDay {
            minutes_of_day: m as u16 + (h * 60) as u16,
        }
    }

    pub fn hours_minutes(self) -> (usize, usize) {
        (
            (self.minutes_of_day / 60) as usize,
            (self.minutes_of_day % 60) as usize,
        )
    }

    pub fn earlier_by(self, delta: Duration) -> Self {
        TimeOfDay {
            minutes_of_day: ((((self.minutes_of_day as isize - delta.as_minutes() as isize)
                % MINUTES_PER_DAY as isize)
                + MINUTES_PER_DAY as isize) as usize
                % MINUTES_PER_DAY) as u16,
        }
    }

    pub fn later_by(self, delta: Duration) -> Self {
        TimeOfDay {
            minutes_of_day: ((self.minutes_of_day as usize + delta.as_minutes() as usize)
                % MINUTES_PER_DAY) as u16,
        }
    }
}

impl ::std::fmt::Debug for TimeOfDay {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(self, f)
    }
}

impl ::std::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let (h, m) = self.hours_minutes();
        write!(f, "{} : {}", h, m)
    }
}

impl From<Instant> for TimeOfDay {
    fn from(instant: Instant) -> TimeOfDay {
        TimeOfDay {
            minutes_of_day: ((BEGINNING_TIME_OF_DAY * 60
                + (instant.ticks() / TICKS_PER_SIM_MINUTE as usize))
                % MINUTES_PER_DAY) as u16,
        }
    }
}

impl<D: Into<Duration>> ::std::ops::Add<D> for TimeOfDay {
    type Output = Self;

    fn add(self, rhs: D) -> Self {
        TimeOfDay {
            minutes_of_day: self.minutes_of_day + (rhs.into().0 / 60) as u16,
        }
    }
}

impl<D: Into<Duration>> ::std::ops::AddAssign<D> for TimeOfDay {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: D) {
        self.minutes_of_day += (rhs.into().0 / 60) as u16
    }
}

impl<D: Into<Duration>> ::std::ops::Sub<D> for TimeOfDay {
    type Output = Self;

    fn sub(self, rhs: D) -> Self {
        TimeOfDay {
            minutes_of_day: self.minutes_of_day - (rhs.into().0 / 60) as u16,
        }
    }
}

impl<D: Into<Duration>> ::std::ops::SubAssign<D> for TimeOfDay {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: D) {
        self.minutes_of_day -= (rhs.into().0 / 60) as u16
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TimeOfDayRange {
    pub start: TimeOfDay,
    pub end: TimeOfDay,
}

impl TimeOfDayRange {
    pub fn new(start_h: usize, start_m: usize, end_h: usize, end_m: usize) -> TimeOfDayRange {
        TimeOfDayRange {
            start: TimeOfDay::new(start_h, start_m),
            end: TimeOfDay::new(end_h, end_m),
        }
    }

    pub fn contains<T: Into<TimeOfDay>>(self, time: T) -> bool {
        let time = time.into();
        if self.start <= self.end {
            self.start <= time && time <= self.end
        } else {
            self.start <= time || time <= self.end
        }
    }

    pub fn earlier_by(self, delta: Duration) -> Self {
        TimeOfDayRange {
            start: self.start.earlier_by(delta),
            end: self.end.earlier_by(delta),
        }
    }

    pub fn later_by(self, delta: Duration) -> Self {
        TimeOfDayRange {
            start: self.start.later_by(delta),
            end: self.end.later_by(delta),
        }
    }

    pub fn end_after_on_same_day(self, time: TimeOfDay) -> bool {
        if self.end > self.start {
            time < self.end
        } else {
            time > self.start || time < self.end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-6, "expected {} ~= {}", a, b);
    }

    #[test]
    fn duration_construction_and_units() {
        let seconds = Duration::from_seconds(90);
        let minutes = Duration::from_minutes(2);
        let hours = Duration::from_hours(1);

        assert_eq!(seconds.0, 90);
        assert_eq!(minutes.0, 120);
        assert_eq!(hours.0, 3_600);

        approx_eq(seconds.as_seconds(), 90.0);
        approx_eq(seconds.as_minutes(), 1.5);
        approx_eq(hours.as_hours(), 1.0);
        approx_eq(hours.as_days(), 1.0 / 24.0);
    }

    #[test]
    fn duration_arithmetic_and_ticks_conversion() {
        let mut total = Duration::from_minutes(1);
        total += Duration::from_seconds(30);
        assert_eq!(total, Duration::from_seconds(90));

        let sum = Duration::from_seconds(20) + Duration::from_seconds(40);
        assert_eq!(sum, Duration::from_seconds(60));

        let ticks: Ticks = Duration::from_seconds(2).into();
        assert_eq!(ticks.0, 2 * TICKS_PER_SIM_SECOND);
    }

    #[test]
    fn instant_arithmetic_with_duration_and_ticks() {
        let mut instant = Instant::new(10);
        assert_eq!(instant.ticks(), 10);
        assert_eq!(instant.iticks(), 10);

        instant += Duration::from_seconds(2);
        assert_eq!(instant.ticks(), 16);

        instant += Ticks(4);
        assert_eq!(instant.ticks(), 20);

        instant -= Duration::from_seconds(1);
        assert_eq!(instant.ticks(), 17);

        instant -= Ticks(2);
        assert_eq!(instant.ticks(), 15);

        let later = Instant::new(100) + Duration::from_seconds(10);
        assert_eq!(later.ticks(), 130);

        let earlier = later - Duration::from_seconds(5);
        assert_eq!(earlier.ticks(), 115);
    }

    #[test]
    fn time_of_day_round_trip_and_wrapping() {
        let morning = TimeOfDay::new(8, 45);
        assert_eq!(morning.hours_minutes(), (8, 45));
        assert_eq!(format!("{}", morning), "8 : 45");
        assert_eq!(format!("{:?}", morning), "8 : 45");

        let wrapped_earlier = TimeOfDay::new(1, 15).earlier_by(Duration::from_hours(2));
        assert_eq!(wrapped_earlier.hours_minutes(), (23, 15));

        let wrapped_later = TimeOfDay::new(23, 50).later_by(Duration::from_minutes(15));
        assert_eq!(wrapped_later.hours_minutes(), (0, 5));
    }

    #[test]
    fn time_of_day_from_instant_and_arithmetic() {
        let start = TimeOfDay::from(Instant::new(0));
        assert_eq!(start.hours_minutes(), (7, 0));

        let one_sim_hour = Instant::new((TICKS_PER_SIM_MINUTE as usize) * 60);
        let hour_later = TimeOfDay::from(one_sim_hour);
        assert_eq!(hour_later.hours_minutes(), (8, 0));

        let mut time = TimeOfDay::new(10, 0);
        time += Duration::from_minutes(30);
        assert_eq!(time.hours_minutes(), (10, 30));

        time -= Duration::from_minutes(15);
        assert_eq!(time.hours_minutes(), (10, 15));

        let added = TimeOfDay::new(12, 0) + Duration::from_minutes(45);
        assert_eq!(added.hours_minutes(), (12, 45));

        let subtracted = added - Duration::from_minutes(30);
        assert_eq!(subtracted.hours_minutes(), (12, 15));
    }

    #[test]
    fn time_of_day_range_contains_and_end_logic() {
        let day_shift = TimeOfDayRange::new(9, 0, 17, 0);
        assert!(day_shift.contains(TimeOfDay::new(9, 0)));
        assert!(day_shift.contains(TimeOfDay::new(12, 0)));
        assert!(day_shift.contains(TimeOfDay::new(17, 0)));
        assert!(!day_shift.contains(TimeOfDay::new(8, 59)));
        assert!(!day_shift.contains(TimeOfDay::new(17, 1)));
        assert!(day_shift.end_after_on_same_day(TimeOfDay::new(16, 59)));
        assert!(!day_shift.end_after_on_same_day(TimeOfDay::new(17, 1)));

        let night_shift = TimeOfDayRange::new(22, 0, 2, 0);
        assert!(night_shift.contains(TimeOfDay::new(23, 30)));
        assert!(night_shift.contains(TimeOfDay::new(1, 30)));
        assert!(!night_shift.contains(TimeOfDay::new(12, 0)));
        assert!(night_shift.end_after_on_same_day(TimeOfDay::new(23, 0)));
        assert!(night_shift.end_after_on_same_day(TimeOfDay::new(1, 0)));
        assert!(!night_shift.end_after_on_same_day(TimeOfDay::new(12, 0)));
    }

    #[test]
    fn time_of_day_range_shifts() {
        let original = TimeOfDayRange::new(10, 0, 12, 0);

        let earlier = original.earlier_by(Duration::from_minutes(30));
        assert_eq!(earlier.start.hours_minutes(), (9, 30));
        assert_eq!(earlier.end.hours_minutes(), (11, 30));

        let later = original.later_by(Duration::from_minutes(45));
        assert_eq!(later.start.hours_minutes(), (10, 45));
        assert_eq!(later.end.hours_minutes(), (12, 45));
    }
}
