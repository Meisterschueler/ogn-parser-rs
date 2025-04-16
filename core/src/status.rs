//! A Status Report announces the station's current mission or any other single
//! line status to everyone. The report starts with the '>' APRS Data Type Identifier.
//! The report may optionally contain a timestamp.
//!
//! Examples:
//! - ">12.6V 0.2A 22degC"              (report without timestamp)
//! - ">120503hFatal error"             (report with timestamp in HMS format)
//! - ">281205zSystem will shutdown"    (report with timestamp in DHM format)

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::Serialize;

use crate::AprsError;
use crate::Timestamp;
use crate::utils::{extract_values, split_value_unit};

#[derive(PartialEq, Debug, Clone, Serialize, Default)]
pub struct AprsStatus {
    pub timestamp: Timestamp,

    pub version: Option<String>,
    pub platform: Option<String>,
    pub cpu_load: Option<f32>,
    pub ram_free: Option<f32>,
    pub ram_total: Option<f32>,
    pub ntp_offset: Option<f32>,
    pub ntp_correction: Option<f32>,
    pub voltage: Option<f32>,
    pub amperage: Option<f32>,
    pub cpu_temperature: Option<f32>,
    pub visible_senders: Option<u16>,
    pub latency: Option<f32>,
    pub senders: Option<u16>,
    pub rf_correction_manual: Option<i16>,
    pub rf_correction_automatic: Option<f32>,
    pub noise: Option<f32>,
    pub senders_signal_quality: Option<f32>,
    pub senders_messages: Option<u32>,
    pub good_senders_signal_quality: Option<f32>,
    pub good_senders: Option<u16>,
    pub good_and_bad_senders: Option<u16>,
    pub unparsed: Option<String>,
}

impl FromStr for AprsStatus {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut status = AprsStatus {
            ..Default::default()
        };

        // Parse timestamp
        status.timestamp = s[0..7]
            .parse::<Timestamp>()
            .map_err(|_| AprsError::InvalidTimestamp(s.to_owned()))?;

        let mut unparsed: Vec<_> = vec![];
        for part in s[7..].split_whitespace() {
            // receiver software version: vX.Y.Z
            // X (major)
            // Y (minor)
            // Z (bugfix)
            if &part[0..1] == "v" && part.matches('.').count() == 3 && status.version.is_none() {
                let (first, second) = part
                    .match_indices('.')
                    .nth(2)
                    .map(|(idx, _)| part.split_at(idx))
                    .unwrap();
                status.version = Some(first[1..].into());
                status.platform = Some(second[1..].into());

            // cpu load: CPU:x.x
            // x.x: cpu load as percentage
            } else if part.len() > 4 && part.starts_with("CPU:") && status.cpu_load.is_none() {
                if let Ok(cpu_load) = part[4..].parse::<f32>() {
                    status.cpu_load = Some(cpu_load);
                } else {
                    unparsed.push(part);
                }

            // RAM usage: RAM:x.x/y.yMB
            // x.x: free RAM in MB
            // y.y: total RAM in MB
            } else if part.len() > 6
                && part.starts_with("RAM:")
                && part.ends_with("MB")
                && part.find('/').is_some()
                && status.ram_free.is_none()
            {
                let subpart = &part[4..part.len() - 2];
                let split_point = subpart.find('/').unwrap();
                let (first, second) = subpart.split_at(split_point);
                let ram_free = first.parse::<f32>().ok();
                let ram_total = second[1..].parse::<f32>().ok();
                if ram_free.is_some() && ram_total.is_some() {
                    status.ram_free = ram_free;
                    status.ram_total = ram_total;
                } else {
                    unparsed.push(part);
                }

            // time synchronisation: NTP:x.xms/y.yppm
            // x.x: NTP offset in [ms]
            // y.y: NTP correction in [ppm]
            } else if part.len() > 6
                && part.starts_with("NTP:")
                && part.find('/').is_some()
                && status.ntp_offset.is_none()
            {
                let subpart = &part[4..part.len() - 3];
                let split_point = subpart.find('/').unwrap();
                let (first, second) = subpart.split_at(split_point);
                let ntp_offset = first[0..first.len() - 2].parse::<f32>().ok();
                let ntp_correction = second[1..].parse::<f32>().ok();
                if ntp_offset.is_some() && ntp_correction.is_some() {
                    status.ntp_offset = ntp_offset;
                    status.ntp_correction = ntp_correction;
                } else {
                    unparsed.push(part);
                }

            // senders count: x/yAcfts[1h]
            // x: visible senders in the last hour
            // y: total senders in the last hour
            } else if part.len() >= 11
                && part.ends_with("Acfts[1h]")
                && part.find('/').is_some()
                && status.visible_senders.is_none()
            {
                let subpart = &part[0..part.len() - 9];
                let split_point = subpart.find('/').unwrap();
                let (first, second) = subpart.split_at(split_point);
                let visible_senders = first.parse::<u16>().ok();
                let senders = second[1..].parse::<u16>().ok();
                if visible_senders.is_some() && senders.is_some() {
                    status.visible_senders = visible_senders;
                    status.senders = senders;
                } else {
                    unparsed.push(part);
                }

            // latency: Lat:x.xs
            // x.x: latency in [s]
            } else if part.len() > 5
                && part.starts_with("Lat:")
                && part.ends_with("s")
                && status.latency.is_none()
            {
                let latency = part[4..part.len() - 1].parse::<f32>().ok();
                if latency.is_some() {
                    status.latency = latency;
                } else {
                    unparsed.push(part);
                }

            // radio frequency informations start with "RF:"
            } else if part.len() >= 11
                && part.starts_with("RF:")
                && status.rf_correction_manual.is_none()
            {
                let values = extract_values(part);
                // short RF format: RF:+x.x/y.yppm/+z.zdB
                // x.x: manual correction in [ppm]
                // y.y: automatic correction in [ppm]
                // z.z: background noise in [dB]
                if values.len() == 3 {
                    let rf_correction_manual = values[0].parse::<i16>().ok();
                    let rf_correction_automatic = values[1].parse::<f32>().ok();
                    let noise = values[2].parse::<f32>().ok();

                    if rf_correction_manual.is_some()
                        && rf_correction_automatic.is_some()
                        && noise.is_some()
                    {
                        status.rf_correction_manual = rf_correction_manual;
                        status.rf_correction_automatic = rf_correction_automatic;
                        status.noise = noise;
                    } else {
                        unparsed.push(part);
                        continue;
                    }
                // medium RF format: RF:+x.x/y.yppm/+z.zdB/+a.adB@10km[b]
                // a.a: sender signal quality [dB]
                // b: number of messages
                } else if values.len() == 6 {
                    let rf_correction_manual = values[0].parse::<i16>().ok();
                    let rf_correction_automatic = values[1].parse::<f32>().ok();
                    let noise = values[2].parse::<f32>().ok();
                    let senders_signal_quality = values[3].parse::<f32>().ok();
                    let senders_messages = values[5].parse::<u32>().ok();
                    if rf_correction_manual.is_some()
                        && rf_correction_automatic.is_some()
                        && noise.is_some()
                        && senders_signal_quality.is_some()
                        && senders_messages.is_some()
                    {
                        status.rf_correction_manual = rf_correction_manual;
                        status.rf_correction_automatic = rf_correction_automatic;
                        status.noise = noise;
                        status.senders_signal_quality = senders_signal_quality;
                        status.senders_messages = senders_messages;
                    } else {
                        unparsed.push(part);
                        continue;
                    }
                // long RF format: RF:+x.x/y.yppm/+z.zdB/+a.adB@10km[b]/+c.cdB@10km[d/e]
                // c.c: good senders signal quality [dB]
                // d: number of good senders
                // e: number of good and bad senders
                } else if values.len() == 10 {
                    let rf_correction_manual = values[0].parse::<i16>().ok();
                    let rf_correction_automatic = values[1].parse::<f32>().ok();
                    let noise = values[2].parse::<f32>().ok();
                    let senders_signal_quality = values[3].parse::<f32>().ok();
                    let senders_messages = values[5].parse::<u32>().ok();
                    let good_senders_signal_quality = values[6].parse::<f32>().ok();
                    let good_senders = values[8].parse::<u16>().ok();
                    let good_and_bad_senders = values[9].parse::<u16>().ok();
                    if rf_correction_manual.is_some()
                        && rf_correction_automatic.is_some()
                        && noise.is_some()
                        && senders_signal_quality.is_some()
                        && senders_messages.is_some()
                        && good_senders_signal_quality.is_some()
                        && good_senders.is_some()
                        && good_and_bad_senders.is_some()
                    {
                        status.rf_correction_manual = rf_correction_manual;
                        status.rf_correction_automatic = rf_correction_automatic;
                        status.noise = noise;
                        status.senders_signal_quality = senders_signal_quality;
                        status.senders_messages = senders_messages;
                        status.good_senders_signal_quality = good_senders_signal_quality;
                        status.good_senders = good_senders;
                        status.good_and_bad_senders = good_and_bad_senders;
                    } else {
                        unparsed.push(part);
                        continue;
                    }
                } else {
                    unparsed.push(part);
                    continue;
                }
            } else if let Some((value, unit)) = split_value_unit(part) {
                // cpu temperature: +x.xC
                // x.x: cpu temperature in [°C]
                if unit == "C" && status.cpu_temperature.is_none() {
                    status.cpu_temperature = value.parse::<f32>().ok();
                // voltage: +x.xV
                // x.x: voltage in [V]
                } else if unit == "V" && status.voltage.is_none() {
                    status.voltage = value.parse::<f32>().ok();
                // currency: +x.xA
                // x.x: currency in [A]
                } else if unit == "A" && status.amperage.is_none() {
                    status.amperage = value.parse::<f32>().ok();
                } else {
                    unparsed.push(part);
                }
            } else {
                unparsed.push(part);
            }
        }
        status.unparsed = if !unparsed.is_empty() {
            Some(unparsed.join(" "))
        } else {
            None
        };

        Ok(status)
    }
}

impl Display for AprsStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, ">{}", self.timestamp)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::WriterBuilder;
    use std::io::stdout;

    #[test]
    fn parse_with_timestamp_without_comment() {
        let result = "312359z".parse::<AprsStatus>().unwrap();
        assert_eq!(result.timestamp, Timestamp::DDHHMM(31, 23, 59));
        assert_eq!(result.unparsed, None);
    }

    #[test]
    fn parse_with_timestamp_and_comment() {
        let result = "235959hHi there!".parse::<AprsStatus>().unwrap();
        assert_eq!(result.timestamp, Timestamp::HHMMSS(23, 59, 59));
        assert_eq!(result.unparsed.unwrap(), "Hi there!");
    }

    #[ignore = "status_comment serialization not implemented"]
    #[test]
    fn test_serialize() {
        let aprs_position = "235959hHi there!".parse::<AprsStatus>().unwrap();
        let mut wtr = WriterBuilder::new().from_writer(stdout());
        wtr.serialize(aprs_position).unwrap();
        wtr.flush().unwrap();
    }

    #[test]
    fn test_sdr() {
        let result = r"000000h v0.2.7.RPI-GPU CPU:0.7 RAM:770.2/968.2MB NTP:1.8ms/-3.3ppm +55.7C 7/8Acfts[1h] RF:+54-1.1ppm/-0.16dB/+7.1dB@10km[19481]/+16.8dB@10km[7/13]".parse::<AprsStatus>().unwrap();
        assert_eq!(result.version, Some("0.2.7".into()));
        assert_eq!(result.platform, Some("RPI-GPU".into()));
        assert_eq!(result.cpu_load, Some(0.7));
        assert_eq!(result.ram_free, Some(770.2));
        assert_eq!(result.ram_total, Some(968.2));
        assert_eq!(result.ntp_offset, Some(1.8));
        assert_eq!(result.ntp_correction, Some(-3.3));
        assert_eq!(result.voltage, None);
        assert_eq!(result.amperage, None);
        assert_eq!(result.cpu_temperature, Some(55.7));
        assert_eq!(result.visible_senders, Some(7));
        assert_eq!(result.senders, Some(8));
        assert_eq!(result.rf_correction_manual, Some(54));
        assert_eq!(result.rf_correction_automatic, Some(-1.1));
        assert_eq!(result.noise, Some(-0.16));
        assert_eq!(result.senders_signal_quality, Some(7.1));
        assert_eq!(result.senders_messages, Some(19481));
        assert_eq!(result.good_senders_signal_quality, Some(16.8));
        assert_eq!(result.good_senders, Some(7));
        assert_eq!(result.good_and_bad_senders, Some(13));
        assert_eq!(result.unparsed, None);
    }

    #[test]
    fn test_rf_3() {
        let result = r"000000h RF:+29+0.0ppm/+35.22dB"
            .parse::<AprsStatus>()
            .unwrap();
        assert_eq!(result.rf_correction_manual, Some(29));
        assert_eq!(result.rf_correction_automatic, Some(0.0));
        assert_eq!(result.noise, Some(35.22));
        assert_eq!(result.unparsed, None);
    }

    #[test]
    fn test_rf_6() {
        let result = r"000000h RF:+41+56.0ppm/-1.87dB/+0.1dB@10km[1928]"
            .parse::<AprsStatus>()
            .unwrap();
        assert_eq!(result.rf_correction_manual, Some(41));
        assert_eq!(result.rf_correction_automatic, Some(56.0));
        assert_eq!(result.noise, Some(-1.87));
        assert_eq!(result.senders_signal_quality, Some(0.1));
        assert_eq!(result.senders_messages, Some(1928));
        assert_eq!(result.unparsed, None);
    }

    #[test]
    fn test_rf_10() {
        let result = r"000000h RF:+54-1.1ppm/-0.16dB/+7.1dB@10km[19481]/+16.8dB@10km[7/13]"
            .parse::<AprsStatus>()
            .unwrap();
        assert_eq!(result.rf_correction_manual, Some(54));
        assert_eq!(result.rf_correction_automatic, Some(-1.1));
        assert_eq!(result.noise, Some(-0.16));
        assert_eq!(result.senders_signal_quality, Some(7.1));
        assert_eq!(result.senders_messages, Some(19481));
        assert_eq!(result.good_senders_signal_quality, Some(16.8));
        assert_eq!(result.good_senders, Some(7));
        assert_eq!(result.good_and_bad_senders, Some(13));
        assert_eq!(result.unparsed, None);
    }
}
