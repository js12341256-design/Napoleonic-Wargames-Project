//! Real-time game clock for Grand Campaign 1805.
//!
//! Tracks in-game date and time, with configurable speed settings.
//! Start date: 1 January 1805.

use serde::{Deserialize, Serialize};

/// How many game-hours per tick at each speed level.
const HOURS_PER_TICK: [u32; 6] = [0, 1, 2, 4, 8, 12];

/// Days in each month (non-leap year; 1805 is not a leap year).
const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Game speed settings, from paused to 5x.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameSpeed {
    Paused,
    Speed1,
    Speed2,
    Speed3,
    Speed4,
    Speed5,
}

impl GameSpeed {
    /// Convert a u8 (0–5) to a speed setting.
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Paused,
            1 => Self::Speed1,
            2 => Self::Speed2,
            3 => Self::Speed3,
            4 => Self::Speed4,
            5 => Self::Speed5,
            _ => Self::Speed1,
        }
    }

    /// Hours advanced per tick at this speed.
    pub fn hours_per_tick(self) -> u32 {
        HOURS_PER_TICK[self as usize]
    }
}

/// Real-time game clock tracking the in-game date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameClock {
    pub tick: u64,
    pub day: u32,
    pub month: u8,
    pub year: u16,
    pub speed: GameSpeed,
    pub paused: bool,
    /// Accumulated hours within the current day (0–23).
    hour: u32,
}

impl GameClock {
    /// Create a new clock starting at 1 January 1805.
    pub fn new() -> Self {
        Self {
            tick: 0,
            day: 1,
            month: 1,
            year: 1805,
            speed: GameSpeed::Speed1,
            paused: false,
            hour: 0,
        }
    }

    /// Advance the clock by one tick. Does nothing if paused.
    pub fn advance_tick(&mut self) {
        if self.paused || self.speed == GameSpeed::Paused {
            return;
        }
        self.tick += 1;
        let hours = self.speed.hours_per_tick();
        self.hour += hours;

        while self.hour >= 24 {
            self.hour -= 24;
            self.advance_day();
        }
    }

    fn advance_day(&mut self) {
        self.day += 1;
        let days_in_current = days_in_month(self.month, self.year);
        if self.day > days_in_current {
            self.day = 1;
            self.month += 1;
            if self.month > 12 {
                self.month = 1;
                self.year += 1;
            }
        }
    }

    /// Returns a formatted date string, e.g. "15 March 1805".
    pub fn date_string(&self) -> String {
        let month_name = MONTH_NAMES[(self.month - 1) as usize];
        format!("{} {} {}", self.day, month_name, self.year)
    }

    /// Set game speed from a u8 value (0=Paused, 1–5=speeds).
    pub fn set_speed(&mut self, speed: u8) {
        self.speed = GameSpeed::from_u8(speed);
        if self.speed == GameSpeed::Paused {
            self.paused = true;
        }
    }

    /// Toggle pause state.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

impl Default for GameClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the number of days in a given month/year (handles leap years).
fn days_in_month(month: u8, year: u16) -> u32 {
    if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[(month - 1) as usize]
    }
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clock_starts_at_1805() {
        let clock = GameClock::new();
        assert_eq!(clock.day, 1);
        assert_eq!(clock.month, 1);
        assert_eq!(clock.year, 1805);
        assert_eq!(clock.tick, 0);
        assert!(!clock.paused);
    }

    #[test]
    fn date_string_format() {
        let clock = GameClock::new();
        assert_eq!(clock.date_string(), "1 January 1805");
    }

    #[test]
    fn advance_tick_increments() {
        let mut clock = GameClock::new();
        clock.advance_tick();
        assert_eq!(clock.tick, 1);
        // At Speed1, 1 hour per tick, so still day 1
        assert_eq!(clock.day, 1);
    }

    #[test]
    fn twenty_four_ticks_advance_one_day_at_speed1() {
        let mut clock = GameClock::new();
        for _ in 0..24 {
            clock.advance_tick();
        }
        assert_eq!(clock.day, 2);
        assert_eq!(clock.month, 1);
    }

    #[test]
    fn month_rollover() {
        let mut clock = GameClock::new();
        // Advance 31 days worth of ticks at Speed1
        for _ in 0..(31 * 24) {
            clock.advance_tick();
        }
        assert_eq!(clock.month, 2);
        assert_eq!(clock.day, 1);
    }

    #[test]
    fn year_rollover() {
        let mut clock = GameClock::new();
        // Advance 365 days at Speed5 (12 hours/tick)
        // 365 days = 365 * 24 hours = 8760 hours
        // At Speed5, 12 hours/tick => 730 ticks
        clock.speed = GameSpeed::Speed5;
        for _ in 0..730 {
            clock.advance_tick();
        }
        assert_eq!(clock.year, 1806);
        assert_eq!(clock.month, 1);
        assert_eq!(clock.day, 1);
    }

    #[test]
    fn paused_does_not_advance() {
        let mut clock = GameClock::new();
        clock.paused = true;
        clock.advance_tick();
        assert_eq!(clock.tick, 0);
        assert_eq!(clock.day, 1);
    }

    #[test]
    fn toggle_pause() {
        let mut clock = GameClock::new();
        assert!(!clock.paused);
        clock.toggle_pause();
        assert!(clock.paused);
        clock.toggle_pause();
        assert!(!clock.paused);
    }

    #[test]
    fn set_speed_paused() {
        let mut clock = GameClock::new();
        clock.set_speed(0);
        assert_eq!(clock.speed, GameSpeed::Paused);
        assert!(clock.paused);
    }

    #[test]
    fn speed_from_u8_out_of_range() {
        assert_eq!(GameSpeed::from_u8(99), GameSpeed::Speed1);
    }

    #[test]
    fn leap_year_detection() {
        assert!(!is_leap_year(1805));
        assert!(is_leap_year(1804));
        assert!(!is_leap_year(1800));
        assert!(is_leap_year(2000));
    }
}
