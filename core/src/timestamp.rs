use chrono::{Duration, prelude::*};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::AprsError;
use serde::Serialize;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Timestamp {
    /// Day of month, Hour and Minute in UTC
    DDHHMM(u8, u8, u8),
    /// Hour, Minute and Second in UTC
    HHMMSS(u8, u8, u8),
    /// Unsupported timestamp format
    Unsupported(String),
}

impl FromStr for Timestamp {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let b = s.as_bytes();

        if b.len() != 7 {
            return Err(AprsError::InvalidTimestamp(s.to_owned()));
        }

        let one = s[0..2]
            .parse::<u8>()
            .map_err(|_| AprsError::InvalidTimestamp(s.to_owned()))?;
        let two = s[2..4]
            .parse::<u8>()
            .map_err(|_| AprsError::InvalidTimestamp(s.to_owned()))?;
        let three = s[4..6]
            .parse::<u8>()
            .map_err(|_| AprsError::InvalidTimestamp(s.to_owned()))?;

        Ok(match (b[6] as char, one, two, three) {
            ('z', 0..=31, 0..=23, 0..=59) => Timestamp::DDHHMM(one, two, three),
            ('h', 0..=23, 0..=59, 0..=59) => Timestamp::HHMMSS(one, two, three),
            ('/', _, _, _) => Timestamp::Unsupported(s.to_owned()),
            _ => return Err(AprsError::InvalidTimestamp(s.to_owned())),
        })
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::DDHHMM(d, h, m) => write!(f, "{d:02}{h:02}{m:02}z"),
            Self::HHMMSS(h, m, s) => write!(f, "{h:02}{m:02}{s:02}h"),
            Self::Unsupported(s) => write!(f, "{s}"),
        }
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}

impl Timestamp {
    pub fn to_datetime(&self, reference: &DateTime<Utc>) -> Result<DateTime<Utc>, AprsError> {
        match self {
            Timestamp::HHMMSS(h, m, s) => {
                let time = NaiveTime::from_hms_opt(*h as u32, *m as u32, *s as u32).unwrap();
                let base_date = reference.date_naive();
                let naive = NaiveDateTime::new(base_date, time);
                let datetime: DateTime<Utc> = Utc.from_utc_datetime(&naive);

                match (datetime - reference).num_hours() {
                    -25..=-23 => Ok(datetime + Duration::days(1)),
                    -1..=1 => Ok(datetime),
                    23..=25 => Ok(datetime - Duration::days(1)),
                    _ => Err(AprsError::TimestampOutOfRange(format!(
                        "{datetime} {reference}"
                    ))),
                }
            }
            Timestamp::DDHHMM(_d, h, m) => {
                // FIXME: d is currently not considered. We always use the day of reference
                let time = NaiveTime::from_hms_opt(*h as u32, *m as u32, 0).unwrap();
                let base_date = reference.date_naive();
                let naive = NaiveDateTime::new(base_date, time);
                let datetime: DateTime<Utc> = Utc.from_utc_datetime(&naive);

                match (datetime - reference).num_hours() {
                    -25..=-23 => Ok(datetime + Duration::days(1)),
                    -1..=1 => Ok(datetime),
                    23..=25 => Ok(datetime - Duration::days(1)),
                    _ => Err(AprsError::TimestampOutOfRange(format!(
                        "{datetime} {reference}"
                    ))),
                }
            }
            Timestamp::Unsupported(_s) => {
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use csv::WriterBuilder;
    use std::io::stdout;

    use super::*;

    #[test]
    fn parse_ddhhmm() {
        assert_eq!("311234z".parse(), Ok(Timestamp::DDHHMM(31, 12, 34)));
    }

    #[test]
    fn parse_hhmmss() {
        assert_eq!("123456h".parse(), Ok(Timestamp::HHMMSS(12, 34, 56)));
    }

    #[test]
    fn parse_local_time() {
        assert_eq!(
            "123456/".parse::<Timestamp>(),
            Ok(Timestamp::Unsupported("123456/".to_owned()))
        );
    }

    #[test]
    fn invalid_timestamp() {
        assert_eq!(
            "1234567".parse::<Timestamp>(),
            Err(AprsError::InvalidTimestamp("1234567".to_owned()))
        );
    }

    #[test]
    fn invalid_timestamp2() {
        assert_eq!(
            "123a56z".parse::<Timestamp>(),
            Err(AprsError::InvalidTimestamp("123a56z".to_owned()))
        );
    }

    #[test]
    fn invalid_ddhhmm() {
        assert_eq!(
            "322460z".parse::<Timestamp>(),
            Err(AprsError::InvalidTimestamp("322460z".to_owned()))
        );
    }

    #[test]
    fn invalid_hhmmss() {
        assert!("230000h".parse::<Timestamp>().is_ok(), "23 is a valid hour");
        assert!(
            "005900h".parse::<Timestamp>().is_ok(),
            "59 is a valid minute"
        );
        assert!(
            "000059h".parse::<Timestamp>().is_ok(),
            "59 is a valid second"
        );

        assert!(
            "240000h".parse::<Timestamp>().is_err(),
            "24 is not a valid hour"
        );
        assert!(
            "006000h".parse::<Timestamp>().is_err(),
            "60 is not a valid minute"
        );
        assert!(
            "000060h".parse::<Timestamp>().is_err(),
            "60 is not a valid second"
        );
    }

    #[test]
    fn test_serialize() {
        let timestamp: Timestamp = "311234z".parse().unwrap();
        let mut wtr = WriterBuilder::new().from_writer(stdout());
        wtr.serialize(timestamp).unwrap();
        wtr.flush().unwrap();
    }

    #[test]
    fn test_hhmmss_within_1h() {
        let reference = Utc.with_ymd_and_hms(2025, 4, 25, 23, 55, 7).unwrap();
        let timestamp = Timestamp::HHMMSS(23, 50, 0);
        let target = Utc.with_ymd_and_hms(2025, 4, 25, 23, 50, 0).unwrap();
        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);
    }

    #[test]
    fn test_hhmmss_within_1h_daychange() {
        let reference = Utc.with_ymd_and_hms(2025, 4, 10, 23, 55, 7).unwrap();
        let timestamp = Timestamp::HHMMSS(0, 5, 20);
        let target = Utc.with_ymd_and_hms(2025, 4, 11, 0, 5, 20).unwrap();
        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);

        let reference = Utc.with_ymd_and_hms(2025, 4, 10, 0, 10, 7).unwrap();
        let timestamp = Timestamp::HHMMSS(23, 49, 20);
        let target = Utc.with_ymd_and_hms(2025, 4, 9, 23, 49, 20).unwrap();
        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);
    }

    #[test]
    fn test_hhmmss_within_1h_monthchange() {
        let reference = Utc.with_ymd_and_hms(2025, 3, 31, 23, 55, 7).unwrap();
        let timestamp = Timestamp::HHMMSS(0, 10, 20);
        let target = Utc.with_ymd_and_hms(2025, 4, 1, 0, 10, 20).unwrap();
        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);

        let reference = Utc.with_ymd_and_hms(2025, 4, 1, 0, 10, 7).unwrap();
        let timestamp = Timestamp::HHMMSS(23, 55, 20);
        let target = Utc.with_ymd_and_hms(2025, 3, 31, 23, 55, 20).unwrap();
        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);
    }

    #[test]
    fn test_hhmmss_bad_time_range() {
        let reference = Utc.with_ymd_and_hms(2025, 4, 10, 12, 10, 7).unwrap();
        let timestamp = Timestamp::HHMMSS(23, 49, 20);
        assert!(timestamp.to_datetime(&reference).is_err());
    }

    #[test]
    fn test_ddhhmm_within_1h() {
        let reference = Utc.with_ymd_and_hms(2025, 4, 10, 23, 55, 7).unwrap();
        let timestamp = Timestamp::DDHHMM(10, 23, 50);
        let target = Utc.with_ymd_and_hms(2025, 4, 10, 23, 50, 0).unwrap();

        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);
    }

    #[test]
    fn test_ddhhmm_within_1h_monthchange() {
        let reference = Utc.with_ymd_and_hms(2025, 3, 31, 23, 55, 0).unwrap();
        let timestamp = Timestamp::DDHHMM(1, 0, 10);
        let target = Utc.with_ymd_and_hms(2025, 4, 1, 0, 10, 0).unwrap();
        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);

        let reference = Utc.with_ymd_and_hms(2025, 4, 1, 0, 10, 0).unwrap();
        let timestamp = Timestamp::DDHHMM(31, 23, 55);
        let target = Utc.with_ymd_and_hms(2025, 3, 31, 23, 55, 0).unwrap();
        assert_eq!(timestamp.to_datetime(&reference).unwrap(), target);
    }
}
