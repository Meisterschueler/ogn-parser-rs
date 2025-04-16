use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::AprsError;
use serde::Serialize;

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub enum Timestamp {
    /// Day of month, Hour and Minute in UTC
    DDHHMM(u8, u8, u8),
    /// Hour, Minute and Second in UTC
    HHMMSS(u8, u8, u8),
    /// Unsupported timestamp format
    Unsupported(String),
    /// Default case
    #[default]
    Unknown,
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

        Ok(match b[6] as char {
            'z' => Timestamp::DDHHMM(one, two, three),
            'h' => Timestamp::HHMMSS(one, two, three),
            '/' => Timestamp::Unsupported(s.to_owned()),
            _ => return Err(AprsError::InvalidTimestamp(s.to_owned())),
        })
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::DDHHMM(d, h, m) => write!(f, "{:02}{:02}{:02}z", d, h, m),
            Self::HHMMSS(h, m, s) => write!(f, "{:02}{:02}{:02}h", h, m, s),
            Self::Unsupported(s) => write!(f, "{}", s),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(test)]
mod tests {
    use csv::WriterBuilder;
    use std::io::stdout;

    use super::*;

    #[test]
    fn parse_ddhhmm() {
        assert_eq!("123456z".parse(), Ok(Timestamp::DDHHMM(12, 34, 56)));
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
    fn test_serialize() {
        let timestamp: Timestamp = "123456z".parse().unwrap();
        let mut wtr = WriterBuilder::new().from_writer(stdout());
        wtr.serialize(timestamp).unwrap();
        wtr.flush().unwrap();
    }
}
