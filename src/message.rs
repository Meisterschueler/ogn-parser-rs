use crate::position_comment::*;
use crate::status_comment::*;
use aprs_parser::AprsError;
use aprs_parser::{AprsData, AprsPacket};
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::convert::Infallible;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Message {
    pub raw_string: String,
    pub aprs_packet: Result<AprsPacket, AprsError>,
    pub position_comment: Option<PositionComment>,
    pub status_comment: Option<StatusComment>,
}

impl FromStr for Message {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let aprs_packet = s.parse::<AprsPacket>();

        let (position_comment, status_comment) = match &aprs_packet {
            Ok(packet) => match &packet.data {
                AprsData::Position(position) => {
                    (position.comment.parse::<PositionComment>().ok(), None)
                }
                AprsData::Status(status) => (None, status.comment.parse::<StatusComment>().ok()),
                AprsData::Message(_) | AprsData::Unknown => (None, None),
            },
            Err(_) => (None, None),
        };

        Ok(Message {
            raw_string: s.to_string(),
            aprs_packet,
            position_comment,
            status_comment,
        })
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 4 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Message", 4)?;
        state.serialize_field("raw_string", &self.raw_string)?;

        match &self.aprs_packet {
            Ok(packet) => state.serialize_field("aprs_packet", packet)?,
            Err(e) => state.serialize_field("parser_error", &e.to_string())?,
        }

        state.serialize_field("position_comment", &self.position_comment)?;
        state.serialize_field("status_comment", &self.status_comment)?;
        state.end()
    }
}
