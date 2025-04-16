use std::fmt::Write;
use std::str::FromStr;

use serde::Serialize;

use crate::AprsError;
use crate::EncodeError;
use crate::Timestamp;
use crate::lonlat::{Latitude, Longitude, encode_latitude, encode_longitude};
use crate::position_comment::PositionComment;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct AprsPosition {
    pub timestamp: Timestamp,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub symbol_table: char,
    pub symbol_code: char,
    pub comment: PositionComment,
}

impl FromStr for AprsPosition {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        // check for minimal message length
        if s.len() < 25 {
            return Err(AprsError::InvalidPosition(s.to_owned()));
        };

        // Extract timestamp
        let timestamp = s[0..7].parse()?;

        // parse position
        let mut latitude: Latitude = s[7..15].parse()?;
        let mut longitude: Longitude = s[16..25].parse()?;

        let symbol_table = s.chars().nth(15).unwrap();
        let symbol_code = s.chars().nth(25).unwrap();

        let comment = &s[26..s.len()];

        // parse the comment
        let ogn = comment.parse::<PositionComment>().unwrap();

        // The comment may contain additional position precision information that will be added to the current position
        if let Some(precision) = &ogn.additional_precision {
            *latitude += precision.lat as f64 / 60_000.;
            *longitude += precision.lon as f64 / 60_000.;
        }

        Ok(AprsPosition {
            timestamp,
            latitude,
            longitude,
            symbol_table,
            symbol_code,
            comment: ogn,
        })
    }
}

impl AprsPosition {
    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        write!(buf, "/{}", self.timestamp)?;

        write!(
            buf,
            "{}{}{}{}{:#?}",
            encode_latitude(self.latitude)?,
            self.symbol_table,
            encode_longitude(self.longitude)?,
            self.symbol_code,
            self.comment,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::WriterBuilder;
    use std::io::stdout;

    #[test]
    fn parse_with_timestamp_without_messaging() {
        let result = r"074849h4821.61N\01224.49E^322/103/A=003054"
            .parse::<AprsPosition>()
            .unwrap();
        assert_eq!(result.timestamp, Timestamp::HHMMSS(7, 48, 49));
        assert_relative_eq!(*result.latitude, 48.36016666666667);
        assert_relative_eq!(*result.longitude, 12.408166666666666);
        assert_eq!(result.symbol_table, '\\');
        assert_eq!(result.symbol_code, '^');
        assert_eq!(result.comment.altitude.unwrap(), 003054);
        assert_eq!(result.comment.course.unwrap(), 322);
        assert_eq!(result.comment.speed.unwrap(), 103);
    }

    #[ignore = "position_comment serialization not implemented"]
    #[test]
    fn test_serialize() {
        let aprs_position = r"074849h4821.61N\01224.49E^322/103/A=003054"
            .parse::<AprsPosition>()
            .unwrap();
        let mut wtr = WriterBuilder::new().from_writer(stdout());
        wtr.serialize(aprs_position).unwrap();
        wtr.flush().unwrap();
    }

    #[test]
    fn test_input_string_too_short() {
        let result = "13244".parse::<AprsPosition>();
        assert!(result.is_err(), "Short input string should return an error");
    }
}
