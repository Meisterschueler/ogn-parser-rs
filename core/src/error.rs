#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum AprsError {
    #[error("Empty Callsign: {0}")]
    EmptyCallsign(String),
    #[error("Empty Callsign SSID: {0}")]
    EmptySSID(String),
    #[error("Invalid Callsign SSID: {0}")]
    InvalidSSID(String),
    #[error("Invalid Timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid Position: {0}")]
    InvalidPosition(String),
    #[error("Invalid Latitude: {0}")]
    InvalidLatitude(String),
    #[error("Invalid Longitude: {0}")]
    InvalidLongitude(String),
    #[error("Invalid Packet: {0}")]
    InvalidPacket(String),
    #[error("Invalid Message Destination: {0}")]
    InvalidMessageDestination(String),
    #[error("Invalid Message ID: {0}")]
    InvalidMessageId(String),
    #[error("String contains non-ASCII characters: {0}")]
    InvalidCoding(String),
    #[error("Invalid additional precision: {0}")]
    InvalidAdditionalPrecision(String),
    #[error("Incomplete Comment: {0}")]
    IncompleteComment(String),
    #[error("Invalid Course/Speed/Altitude: {0}")]
    InvalidCourseSpeedAltitude(String),
    #[error("Invalid Altitude: {0}")]
    InvalidAltitude(String),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum EncodeError {
    #[error("Invalid Latitude: {0}")]
    InvalidLatitude(f64),
    #[error("Invalid Longitude: {0}")]
    InvalidLongitude(f64),
    #[error("Invalid Aprs Data")]
    InvalidData,
    #[error(transparent)]
    Format(#[from] std::fmt::Error),
}
