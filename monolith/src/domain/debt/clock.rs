use chrono::{DateTime, NaiveDate, TimeZone, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;

    fn today_date(&self) -> NaiveDate {
        self.now().date_naive()
    }
}

#[derive(Debug, Clone)]
pub struct FixedClock {
    fixed_now: DateTime<Utc>,
}

impl FixedClock {
    pub fn new(fixed_now: DateTime<Utc>) -> Self {
        Self { fixed_now }
    }

    /// Data de referência padrão do sistema: `2024-05-10T00:00:00Z`.
    pub fn default_reference() -> Self {
        let dt = Utc
            .with_ymd_and_hms(2024, 5, 10, 0, 0, 0)
            .single()
            .expect("Valid fixed reference timestamp 2024-05-10T00:00:00Z");
        Self { fixed_now: dt }
    }
}

impl Default for FixedClock {
    fn default() -> Self {
        Self::default_reference()
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.fixed_now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_clock_default_reference_date() {
        let clock = FixedClock::default_reference();
        assert_eq!(
            clock.today_date(),
            NaiveDate::from_ymd_opt(2024, 5, 10).unwrap()
        );
    }
}
