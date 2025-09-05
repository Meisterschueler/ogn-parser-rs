use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::Serialize;

use crate::AprsError;

#[derive(Eq, PartialEq, Debug, Clone, Serialize)]
#[serde(into = "String")]
pub struct Callsign(pub String);

impl From<Callsign> for String {
    fn from(val: Callsign) -> Self {
        val.0
    }
}

impl Callsign {
    pub fn new<T: Into<String>>(call: T) -> Callsign {
        Callsign(call.into())
    }
}

impl FromStr for Callsign {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.is_empty() {
            return Err(AprsError::EmptyCallsign(s.to_owned()));
        }
        Ok(Callsign::new(s))
    }
}

impl Display for Callsign {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_callsign() {
        assert_eq!("ABCDEF".parse(), Ok(Callsign::new("ABCDEF")));
    }

    #[test]
    fn parse_with_ssid() {
        assert_eq!("ABCDEF-42".parse(), Ok(Callsign::new("ABCDEF-42")));
    }

    #[test]
    fn empty_callsign() {
        assert_eq!(
            "".parse::<Callsign>(),
            Err(AprsError::EmptyCallsign("".to_owned()))
        );
    }

    #[test]
    fn parse_callsign_with_dash() {
        assert_eq!("D-EKDF".parse(), Ok(Callsign::new("D-EKDF")));
    }
}
