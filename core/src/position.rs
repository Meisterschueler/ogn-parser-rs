use std::fmt::Write;
use std::str::FromStr;

use serde::Serialize;

use crate::AprsError;
use crate::EncodeError;
use crate::Timestamp;
use crate::lonlat::{Latitude, Longitude, encode_latitude, encode_longitude};
use crate::utils::split_value_unit;

#[derive(PartialEq, Debug, Clone, Serialize, Default)]
pub struct AprsPosition {
    pub timestamp: Timestamp,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub symbol_table: char,
    pub symbol_code: char,

    pub course: Option<u16>,
    pub speed: Option<u16>,
    pub altitude: Option<u32>,
    pub id: Option<ID>,
    pub climb_rate: Option<i16>,
    pub turn_rate: Option<f32>,
    pub signal_quality: Option<f32>,
    pub error: Option<u8>,
    pub frequency_offset: Option<f32>,
    pub gps_quality: Option<String>,
    pub flight_level: Option<f32>,
    pub signal_power: Option<f32>,
    pub software_version: Option<f32>,
    pub hardware_version: Option<u8>,
    pub original_address: Option<u32>,
    pub unparsed: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize)]
pub struct AdditionalPrecision {
    pub lat: u8,
    pub lon: u8,
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize)]
pub struct ID {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved: Option<u16>,
    pub address_type: u16,
    pub aircraft_type: u8,
    pub is_stealth: bool,
    pub is_notrack: bool,
    pub address: u32,
}

impl FromStr for AprsPosition {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut position = AprsPosition {
            ..Default::default()
        };

        // check for minimal message length
        if s.len() < 25 {
            return Err(AprsError::InvalidPosition(s.to_owned()));
        };

        // Extract timestamp
        position.timestamp = s[0..7].parse()?;

        // parse position
        position.latitude = s[7..15].parse()?;
        position.longitude = s[16..25].parse()?;

        // parse symbol table and code
        position.symbol_table = s.chars().nth(15).unwrap();
        position.symbol_code = s.chars().nth(25).unwrap();

        // get the comment
        let comment = &s[26..];
        let parts = comment.split_ascii_whitespace().collect::<Vec<_>>();

        // at least the first part with the altitude is required
        if parts.is_empty() {
            return Err(AprsError::IncompleteComment(comment.to_owned()));
        }

        // parse the comment
        let mut unparsed: Vec<_> = vec![];
        for (idx, part) in parts.into_iter().enumerate() {
            if idx == 0 {
                // The first part can be course + speed + altitude: ccc/sss/A=aaaaaa
                // ccc: course in degrees 0-360
                // sss: speed in km/h
                // aaaaaa: altitude in feet
                if part.len() == 16 {
                    let subparts = part.split('/').collect::<Vec<_>>();
                    if subparts.len() != 3 {
                        return Err(AprsError::InvalidCourseSpeedAltitude(part.to_owned()));
                    }
                    position.course = subparts[0]
                        .parse::<u16>()
                        .map_err(|_| AprsError::InvalidCourseSpeedAltitude(part.to_owned()))
                        .ok();
                    position.speed = subparts[1]
                        .parse::<u16>()
                        .map_err(|_| AprsError::InvalidCourseSpeedAltitude(part.to_owned()))
                        .ok();
                    if &subparts[2][0..2] == "A=" {
                        position.altitude = subparts[2][2..]
                            .parse::<u32>()
                            .map_err(|_| AprsError::InvalidCourseSpeedAltitude(part.to_owned()))
                            .ok();
                    } else {
                        return Err(AprsError::InvalidCourseSpeedAltitude(part.to_owned()));
                    }
                // ... or just the altitude: /A=aaaaaa
                // aaaaaa: altitude in feet
                } else if part.len() == 9 && &part[0..3] == "/A=" {
                    position.altitude = part[3..]
                        .parse::<u32>()
                        .map_err(|_| AprsError::InvalidAltitude(part.to_owned()))
                        .ok();
                } else {
                    return Err(AprsError::InvalidCourseSpeedAltitude(part.to_owned()));
                }

            // The second part could be the additional precision: !Wab!
            // a: additional latitude precision
            // b: additional longitude precision
            } else if idx == 1 && part.len() == 5 && &part[0..2] == "!W" && &part[4..] == "!" {
                *position.latitude += part[2..3]
                    .parse::<u8>()
                    .map_err(|_| AprsError::InvalidAdditionalPrecision(part.to_owned()))
                    .unwrap() as f64
                    / 60_000.;
                *position.longitude += part[3..4]
                    .parse::<u8>()
                    .map_err(|_| AprsError::InvalidAdditionalPrecision(part.to_owned()))
                    .unwrap() as f64
                    / 60_000.;
            // generic ID format: idXXYYYYYY (4 bytes format)
            // YYYYYY: 24 bit address in hex digits
            // XX in hex digits encodes stealth mode, no-tracking flag and address type
            // XX to binary-> STtt ttaa
            // S: stealth flag
            // T: no-tracking flag
            // tttt: aircraft type
            // aa: address type
            } else if part.len() == 10 && &part[0..2] == "id" && position.id.is_none() {
                if let (Some(detail), Some(address)) = (
                    u8::from_str_radix(&part[2..4], 16).ok(),
                    u32::from_str_radix(&part[4..10], 16).ok(),
                ) {
                    let address_type = (detail & 0b0000_0011) as u16;
                    let aircraft_type = (detail & 0b_0011_1100) >> 2;
                    let is_notrack = (detail & 0b0100_0000) != 0;
                    let is_stealth = (detail & 0b1000_0000) != 0;
                    position.id = Some(ID {
                        address_type,
                        aircraft_type,
                        is_notrack,
                        is_stealth,
                        address,
                        ..Default::default()
                    });
                } else {
                    unparsed.push(part);
                }
            // NAVITER ID format: idXXXXYYYYYY (5 bytes)
            // YYYYYY: 24 bit address in hex digits
            // XXXX in hex digits encodes stealth mode, no-tracking flag and address type
            // XXXX to binary-> STtt ttaa aaaa rrrr
            // S: stealth flag
            // T: no-tracking flag
            // tttt: aircraft type
            // aaaaaa: address type
            // rrrr: (reserved)
            } else if part.len() == 12 && &part[0..2] == "id" && position.id.is_none() {
                if let (Some(detail), Some(address)) = (
                    u16::from_str_radix(&part[2..6], 16).ok(),
                    u32::from_str_radix(&part[6..12], 16).ok(),
                ) {
                    let reserved = detail & 0b0000_0000_0000_1111;
                    let address_type = (detail & 0b0000_0011_1111_0000) >> 4;
                    let aircraft_type = ((detail & 0b0011_1100_0000_0000) >> 10) as u8;
                    let is_notrack = (detail & 0b0100_0000_0000_0000) != 0;
                    let is_stealth = (detail & 0b1000_0000_0000_0000) != 0;
                    position.id = Some(ID {
                        reserved: Some(reserved),
                        address_type,
                        aircraft_type,
                        is_notrack,
                        is_stealth,
                        address,
                    });
                } else {
                    unparsed.push(part);
                }
            } else if let Some((value, unit)) = split_value_unit(part) {
                if unit == "fpm" && position.climb_rate.is_none() {
                    position.climb_rate = value.parse::<i16>().ok();
                } else if unit == "rot" && position.turn_rate.is_none() {
                    position.turn_rate = value.parse::<f32>().ok();
                } else if unit == "dB" && position.signal_quality.is_none() {
                    position.signal_quality = value.parse::<f32>().ok();
                } else if unit == "kHz" && position.frequency_offset.is_none() {
                    position.frequency_offset = value.parse::<f32>().ok();
                } else if unit == "e" && position.error.is_none() {
                    position.error = value.parse::<u8>().ok();
                } else if unit == "dBm" && position.signal_power.is_none() {
                    position.signal_power = value.parse::<f32>().ok();
                } else {
                    unparsed.push(part);
                }
            // Gps precision: gpsAxB
            // A: integer
            // B: integer
            } else if part.len() >= 6 && &part[0..3] == "gps" && position.gps_quality.is_none() {
                if let Some((first, second)) = part[3..].split_once('x') {
                    if first.parse::<u8>().is_ok() && second.parse::<u8>().is_ok() {
                        position.gps_quality = Some(part[3..].to_string());
                    } else {
                        unparsed.push(part);
                    }
                } else {
                    unparsed.push(part);
                }
            // Flight level: FLxx.yy
            // xx.yy: float value for flight level
            } else if part.len() >= 3 && &part[0..2] == "FL" && position.flight_level.is_none() {
                if let Ok(flight_level) = part[2..].parse::<f32>() {
                    position.flight_level = Some(flight_level);
                } else {
                    unparsed.push(part);
                }
            // Software version: sXX.YY
            // XX.YY: float value for software version
            } else if part.len() >= 2 && &part[0..1] == "s" && position.software_version.is_none() {
                if let Ok(software_version) = part[1..].parse::<f32>() {
                    position.software_version = Some(software_version);
                } else {
                    unparsed.push(part);
                }
            // Hardware version: hXX
            // XX: hexadecimal value for hardware version
            } else if part.len() == 3 && &part[0..1] == "h" && position.hardware_version.is_none() {
                if part[1..3].chars().all(|c| c.is_ascii_hexdigit()) {
                    position.hardware_version = u8::from_str_radix(&part[1..3], 16).ok();
                } else {
                    unparsed.push(part);
                }
            // Original address: rXXXXXX
            // XXXXXX: hex digits for 24 bit address
            } else if part.len() == 7 && &part[0..1] == "r" && position.original_address.is_none() {
                if part[1..7].chars().all(|c| c.is_ascii_hexdigit()) {
                    position.original_address = u32::from_str_radix(&part[1..7], 16).ok();
                } else {
                    unparsed.push(part);
                }
            } else {
                unparsed.push(part);
            }
        }
        position.unparsed = if !unparsed.is_empty() {
            Some(unparsed.join(" "))
        } else {
            None
        };

        Ok(position)
    }
}

impl AprsPosition {
    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        write!(buf, "/{}", self.timestamp)?;

        write!(
            buf,
            "{}{}{}{}",
            encode_latitude(self.latitude)?,
            self.symbol_table,
            encode_longitude(self.longitude)?,
            self.symbol_code,
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
        assert_eq!(result.altitude.unwrap(), 003054);
        assert_eq!(result.course.unwrap(), 322);
        assert_eq!(result.speed.unwrap(), 103);
    }

    #[ignore = "position serialization not implemented"]
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

    #[test]
    fn test_flr_comment() {
        let result = r"012345h2356.79N\09876.54E^255/045/A=003399 !W03! id06DDFAA3 -613fpm -3.9rot 22.5dB 7e -7.0kHz gps3x7 s7.07 h41 rD002F8".parse::<AprsPosition>().unwrap();
        assert_eq!(result.course, Some(255));
        assert_eq!(result.speed, Some(45));
        assert_eq!(result.altitude, Some(3399));
        assert_eq!(result.id.is_some(), true);
        let id = result.id.unwrap();
        assert_eq!(id.reserved, None);
        assert_eq!(id.address_type, 2);
        assert_eq!(id.aircraft_type, 1);
        assert_eq!(id.is_stealth, false);
        assert_eq!(id.is_notrack, false);
        assert_eq!(id.address, u32::from_str_radix("DDFAA3", 16).unwrap());
        assert_eq!(result.climb_rate, Some(-613));
        assert_eq!(result.turn_rate, Some(-3.9));
        assert_eq!(result.signal_quality, Some(22.5));
        assert_eq!(result.error, Some(7));
        assert_eq!(result.frequency_offset, Some(-7.0));
        assert_eq!(result.gps_quality, Some("3x7".into()));
        assert_eq!(result.software_version, Some(7.07));
        assert_eq!(result.hardware_version, Some(65));
        assert_eq!(
            result.original_address,
            u32::from_str_radix("D002F8", 16).ok()
        );
    }

    #[test]
    fn test_naviter_id() {
        let result = r"012345h2356.79N\09876.54E^000/000/A=000000 !W00! id985F579BDF"
            .parse::<AprsPosition>()
            .unwrap();
        assert_eq!(result.id.is_some(), true);
        let id = result.id.unwrap();

        assert_eq!(id.reserved, Some(15));
        assert_eq!(id.address_type, 5);
        assert_eq!(id.aircraft_type, 6);
        assert_eq!(id.is_stealth, true);
        assert_eq!(id.is_notrack, false);
        assert_eq!(id.address, 0x579BDF);
    }
}
