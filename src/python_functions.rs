use crate::message::*;
use aprs_parser::{AprsData, Callsign};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyList;
use std::collections::HashMap;

#[pyfunction]
pub fn parse_to_json(py: Python<'_>, o: PyObject) -> PyResult<PyObject> {
    if let Ok(s) = o.extract::<&str>(py) {
        let message = s.parse::<Message>().unwrap();
        let result = serde_json::to_string(&message).unwrap();
        Ok(result.into_py(py))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected a string",
        ))
    }
}

#[pyfunction]
pub fn parse(py: Python<'_>, o: PyObject) -> PyResult<PyObject> {
    if let Ok(s) = o.extract::<&str>(py) {
        parse_str(py, s)
    } else if let Ok(list) = o.downcast_bound::<PyList>(py) {
        let results = list
            .iter()
            .map(|item| {
                let s = item
                    .extract::<&str>()
                    .expect("List contains non-string elements");
                parse_str(py, s).unwrap()
            })
            .collect::<Vec<_>>();
        Ok(results.into_py(py))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected a string or a list of strings",
        ))
    }
}

fn parse_str(py: Python<'_>, s: &str) -> PyResult<PyObject> {
    let mut result: HashMap<String, PyObject> = HashMap::new();

    let message = s.parse::<Message>().unwrap();
    result.insert("raw_message".to_string(), message.raw_string.into_py(py));

    if let Ok(aprs_packet) = message.aprs_packet {
        result.insert("from".to_string(), aprs_packet.from.to_string().into_py(py));
        result.insert("to".to_string(), aprs_packet.to.to_string().into_py(py));
        result.insert(
            "via".to_string(),
            aprs_packet
                .via
                .iter()
                .map(Callsign::to_string)
                .collect::<Vec<_>>()
                .into_py(py),
        );

        match aprs_packet.data {
            AprsData::Position(position) => {
                let mut aprs_data: HashMap<String, PyObject> = HashMap::new();

                position.timestamp.map(|timestamp| {
                    aprs_data.insert("timestamp".to_string(), timestamp.to_string().into_py(py))
                });
                aprs_data.insert("latitude".to_string(), position.latitude.into_py(py));
                aprs_data.insert("longitude".to_string(), position.longitude.into_py(py));
                aprs_data.insert(
                    "symbol_table".to_string(),
                    position.symbol_table.into_py(py),
                );
                aprs_data.insert("symbol_code".to_string(), position.symbol_code.into_py(py));
                aprs_data.insert("comment".to_string(), position.comment.clone().into_py(py));

                result.insert(
                    "position".to_string(),
                    aprs_data.into_py_dict_bound(py).into(),
                );
            }
            AprsData::Status(status) => {
                let mut aprs_data: HashMap<String, PyObject> = HashMap::new();

                status.timestamp.map(|timestamp| {
                    aprs_data.insert("timestamp".to_string(), timestamp.to_string().into_py(py))
                });

                aprs_data.insert("comment".to_string(), status.comment.clone().into_py(py));

                result.insert(
                    "status".to_string(),
                    aprs_data.into_py_dict_bound(py).into(),
                );
            }
            AprsData::Message(_) | AprsData::Unknown => {
                result.insert("message_type".to_string(), "unknown".into_py(py));
            }
        }
    }

    if let Some(position_comment) = message.position_comment {
        let mut comment: HashMap<String, PyObject> = HashMap::new();
        position_comment
            .course
            .map(|course| comment.insert("course".to_string(), course.into_py(py)));
        position_comment
            .speed
            .map(|speed| comment.insert("speed".to_string(), speed.into_py(py)));
        position_comment
            .altitude
            .map(|altitude| comment.insert("altitude".to_string(), altitude.into_py(py)));
        if let Some(additional_precision) = position_comment.additional_precision {
            comment.insert(
                "additional_precision".to_string(),
                vec![
                    ("lat".to_string(), additional_precision.lat.into_py(py)),
                    ("lon".to_string(), additional_precision.lon.into_py(py)),
                ]
                .into_py_dict_bound(py)
                .into_py(py),
            );
        }
        if let Some(id) = position_comment.id {
            comment.insert(
                "id".to_string(),
                vec![
                    ("address_type".to_string(), id.address_type.into_py(py)),
                    ("aircraft_type".to_string(), id.aircraft_type.into_py(py)),
                    ("is_stealth".to_string(), id.is_stealth.into_py(py)),
                    ("is_notrack".to_string(), id.is_notrack.into_py(py)),
                    ("address".to_string(), id.address.into_py(py)),
                ]
                .into_py_dict_bound(py)
                .into_py(py),
            );
        }
        position_comment
            .climb_rate
            .map(|climb_rate| comment.insert("climb_rate".to_string(), climb_rate.into_py(py)));
        position_comment
            .turn_rate
            .map(|turn_rate| comment.insert("turn_rate".to_string(), turn_rate.into_py(py)));
        position_comment.signal_quality.map(|signal_quality| {
            comment.insert("signal_quality".to_string(), signal_quality.into_py(py))
        });
        position_comment
            .error
            .map(|error| comment.insert("error".to_string(), error.into_py(py)));
        position_comment.frequency_offset.map(|frequency_offset| {
            comment.insert("frequency_offset".to_string(), frequency_offset.into_py(py))
        });
        position_comment
            .gps_quality
            .map(|gps_quality| comment.insert("gps_quality".to_string(), gps_quality.into_py(py)));
        position_comment.flight_level.map(|flight_level| {
            comment.insert("flight_level".to_string(), flight_level.into_py(py))
        });
        position_comment.signal_power.map(|signal_power| {
            comment.insert("signal_power".to_string(), signal_power.into_py(py))
        });
        position_comment.software_version.map(|software_version| {
            comment.insert("software_version".to_string(), software_version.into_py(py))
        });
        position_comment.hardware_version.map(|hardware_version| {
            comment.insert("hardware_version".to_string(), hardware_version.into_py(py))
        });
        position_comment.original_address.map(|original_address| {
            comment.insert("original_address".to_string(), original_address.into_py(py))
        });
        position_comment
            .unparsed
            .map(|unparsed| comment.insert("unparsed".to_string(), unparsed.into_py(py)));

        result.insert("ogn".to_string(), comment.into_py_dict_bound(py).into());
    }

    if let Some(status_comment) = message.status_comment {
        let mut comment: HashMap<String, PyObject> = HashMap::new();
        status_comment
            .version
            .map(|version| comment.insert("version".to_string(), version.into_py(py)));
        status_comment
            .platform
            .map(|platform| comment.insert("platform".to_string(), platform.into_py(py)));
        status_comment
            .cpu_load
            .map(|cpu_load| comment.insert("cpu_load".to_string(), cpu_load.into_py(py)));
        status_comment
            .ram_free
            .map(|ram_free| comment.insert("ram_free".to_string(), ram_free.into_py(py)));
        status_comment
            .ram_total
            .map(|ram_total| comment.insert("ram_total".to_string(), ram_total.into_py(py)));
        status_comment
            .ntp_offset
            .map(|ntp_offset| comment.insert("ntp_offset".to_string(), ntp_offset.into_py(py)));
        status_comment.ntp_correction.map(|ntp_correction| {
            comment.insert("ntp_correction".to_string(), ntp_correction.into_py(py))
        });
        status_comment
            .voltage
            .map(|voltage| comment.insert("voltage".to_string(), voltage.into_py(py)));
        status_comment
            .amperage
            .map(|amperage| comment.insert("amperage".to_string(), amperage.into_py(py)));
        status_comment.cpu_temperature.map(|cpu_temperature| {
            comment.insert("cpu_temperature".to_string(), cpu_temperature.into_py(py))
        });
        status_comment.visible_senders.map(|visible_senders| {
            comment.insert("visible_senders".to_string(), visible_senders.into_py(py))
        });
        status_comment
            .latency
            .map(|latency| comment.insert("latency".to_string(), latency.into_py(py)));
        status_comment
            .senders
            .map(|senders| comment.insert("senders".to_string(), senders.into_py(py)));
        status_comment
            .rf_correction_manual
            .map(|rf_correction_manual| {
                comment.insert(
                    "rf_correction_manual".to_string(),
                    rf_correction_manual.into_py(py),
                )
            });
        status_comment
            .rf_correction_automatic
            .map(|rf_correction_automatic| {
                comment.insert(
                    "rf_correction_automatic".to_string(),
                    rf_correction_automatic.into_py(py),
                )
            });
        status_comment
            .noise
            .map(|noise| comment.insert("noise".to_string(), noise.into_py(py)));
        status_comment
            .senders_signal_quality
            .map(|senders_signal_quality| {
                comment.insert(
                    "senders_signal_quality".to_string(),
                    senders_signal_quality.into_py(py),
                )
            });
        status_comment.senders_messages.map(|senders_messages| {
            comment.insert("senders_messages".to_string(), senders_messages.into_py(py))
        });
        status_comment
            .good_senders_signal_quality
            .map(|good_senders_signal_quality| {
                comment.insert(
                    "good_senders_signal_quality".to_string(),
                    good_senders_signal_quality.into_py(py),
                )
            });
        status_comment.good_senders.map(|good_senders| {
            comment.insert("good_senders".to_string(), good_senders.into_py(py))
        });
        status_comment
            .good_and_bad_senders
            .map(|good_and_bad_senders| {
                comment.insert(
                    "good_and_bad_senders".to_string(),
                    good_and_bad_senders.into_py(py),
                )
            });
        status_comment
            .unparsed
            .map(|unparsed| comment.insert("unparsed".to_string(), unparsed.into_py(py)));

        result.insert("ogn".to_string(), comment.into_py_dict_bound(py).into());
    }

    Ok(result.into_py(py))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::IntoPyDict;
    use pyo3::types::PyDict;

    fn compare_dicts(py: Python<'_>, actual: &Bound<PyDict>, expected: &Bound<PyDict>) {
        for (k, v) in expected.iter() {
            let result_value = actual
                .get_item(&k)
                .expect(&format!("Key {k} not found!\nGot: {actual:?}"))
                .expect(&format!("Key {k} not found!\nGot: {actual:?}"));
            if let (Ok(dict1), Ok(dict2)) =
                (v.downcast::<PyDict>(), result_value.downcast::<PyDict>())
            {
                compare_dicts(py, dict1, dict2);
            } else {
                assert!(
                    result_value.eq(&v).unwrap(),
                    "Failed on key: {k}\nExpected: {v:?}\nGot: {result_value:?}"
                );
            }
        }
    }

    #[test]
    fn test_parse_single_string() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let result = parse(py, "test string".to_object(py)).unwrap();
            let result_dict = result.downcast_bound::<PyDict>(py).unwrap();

            assert_eq!(
                result_dict
                    .get_item("raw_message")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "test string"
            );
        });
    }

    #[test]
    fn test_parse_list_of_strings() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let strings = vec!["string1", "string2", "string3"];
            let result = parse(py, strings.to_object(py)).unwrap();
            let result_list: Vec<HashMap<String, PyObject>> = result.extract(py).unwrap();

            assert_eq!(result_list.len(), 3);
            assert_eq!(
                result_list[0]
                    .get("raw_message")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap(),
                "string1"
            );
            assert_eq!(
                result_list[1]
                    .get("raw_message")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap(),
                "string2"
            );
            assert_eq!(
                result_list[2]
                    .get("raw_message")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap(),
                "string3"
            );
        });
    }

    #[test]
    fn test_parse_position() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let result = parse(py, r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1 Mahlzeit!".to_string().into_py(py)).unwrap();
            let result_dict = result.downcast_bound::<PyDict>(py).unwrap();

            let expected = vec![
                ("from", "ICA3D17F2".into_py(py)),
                ("to", "APRS".into_py(py)),
                ("via", vec!["qAS", "dl4mea"].into_py(py)),
                (
                    "position",
                    vec![
                        ("timestamp", "074849h".into_py(py)),
                        ("latitude", 48.36016666666667.into_py(py)),
                        ("symbol_table", '\\'.into_py(py)),
                        ("longitude", 12.408166666666666.into_py(py)),
                        ("symbol_code", '^'.into_py(py)),
                        ("comment", "322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1 Mahlzeit!".into_py(py))
                    ]
                    .into_py_dict_bound(py)
                    .into(),
                ),
                (
                    "ogn",
                    vec![
                        ("course", 322.into_py(py)),
                        ("speed", 103.into_py(py)),
                        ("altitude", 3054.into_py(py)),
                        (
                            "additional_precision",
                            vec![("lat", 0.into_py(py)), ("lon", 9.into_py(py))]
                                .into_py_dict_bound(py)
                                .into(),
                        ),
                        (
                            "id",
                            [
                                ("address_type", 1.into_py(py)),
                                ("aircraft_type", 8.into_py(py)),
                                ("is_stealth", false.into_py(py)),
                                ("is_notrack", false.into_py(py)),
                                ("address", 0x3D17F2.into_py(py)),
                            ]
                            .into_py_dict_bound(py)
                            .into(),
                        ),
                        ("climb_rate", (-039).into_py(py)),
                        ("turn_rate", (0.0).into_py(py)),
                        ("signal_quality", 2.5.into_py(py)),
                        ("error", 3.into_py(py)),
                        ("frequency_offset", (0.0).into_py(py)),
                        ("gps_quality", "1x1".into_py(py)),
                        ("unparsed", "Mahlzeit!".into_py(py)),
                    ]
                    .into_py_dict_bound(py)
                    .into(),
                ),
            ]
            .into_py_dict_bound(py);

            compare_dicts(py, result_dict, &expected);
        });
    }
    /*
    #[test]
    fn parse_error_no_ascii() {
        let result =
            r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054 Hochkönig"
                .parse::<AprsPacket>();
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn parse_message() {
        let result =
            r"ICA3D17F2>Aprs,qAS,dl4mea::DEST     :Hello World! This msg has a : colon {32975"
                .parse::<AprsPacket>()
                .unwrap();
        assert_eq!(result.from, Callsign::new("ICA3D17F2", None));
        assert_eq!(result.to, Callsign::new("Aprs", None));
        assert_eq!(
            result.via,
            vec![Callsign::new("qAS", None), Callsign::new("dl4mea", None),]
        );

        match result.data {
            AprsData::Message(msg) => {
                assert_eq!(msg.addressee, "DEST");
                assert_eq!(msg.text, "Hello World! This msg has a : colon ");
                assert_eq!(msg.id, Some(32975));
            }
            _ => panic!("Unexpected data type"),
        }
    }
    */

    #[test]
    fn test_parse_status() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let result = parse(
                py,
                r"ICA3D17F2>APRS,qAS,dl4mea:>312359zStatus seems okay!"
                    .to_string()
                    .into_py(py),
            )
            .unwrap();
            let result_dict = result.downcast_bound::<PyDict>(py).unwrap();

            let expected = vec![
                ("from", "ICA3D17F2".into_py(py)),
                ("to", "APRS".into_py(py)),
                ("via", vec!["qAS", "dl4mea"].into_py(py)),
                (
                    "status",
                    vec![
                        ("timestamp", "312359z".to_string().into_py(py)),
                        ("unparsed", "Status seems okay!".to_string().into_py(py)),
                        ("comment", "Status seems okay!".into_py(py)),
                    ]
                    .into_py_dict_bound(py)
                    .into(),
                ),
            ]
            .into_py_dict_bound(py);

            compare_dicts(py, result_dict, &expected);
        });
    }
}
