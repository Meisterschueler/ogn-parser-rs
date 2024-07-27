mod message;
mod position_comment;
mod python_functions;
mod status_comment;
mod utils;

use crate::python_functions::{parse, parse_to_json};
use pyo3::prelude::*;

pub use message::Message;

#[pymodule]
fn ognparser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_to_json, m)?)?;
    Ok(())
}
