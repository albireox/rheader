/*
 *  @Author: José Sánchez-Gallego (gallegoj@uw.edu)
 *  @Date: 2025-12-10
 *  @Filename: python.rs
 *  @License: BSD 3-clause (http://www.opensource.org/licenses/BSD-3-Clause)
 */

use pyo3::{IntoPyObjectExt, prelude::*};

use pyo3::exceptions::PyIOError;
use pyo3::types::{PyDict, PyString};

use crate::header::read_header;

// Python class wrapper for Header.
#[pyclass]
struct Header {
    #[pyo3(get)]
    keywords: Py<PyDict>,
}

// Implement __repr__ for Header.
#[pymethods]
impl Header {
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Ok(format!(
            "<Header with {} keywords>",
            slf.borrow().keywords.bind(slf.py()).len()
        ))
    }
}

// Python class wrapper for Keyword.
#[pyclass]
struct Keyword {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    value: Py<PyAny>,
    #[pyo3(get)]
    comment: Option<Py<PyAny>>,
}

// Implement __repr__ for Keyword.
#[pymethods]
impl Keyword {
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let s_borrow = slf.borrow();

        if s_borrow.comment.is_none() {
            return Ok(format!("('{}', {})", s_borrow.name, s_borrow.value));
        } else {
            if s_borrow.value.bind(slf.py()).is_instance_of::<PyString>() {
                Ok(format!(
                    "('{}', '{}', '{}')",
                    s_borrow.name,
                    s_borrow.value,
                    s_borrow.comment.as_ref().unwrap()
                ))
            } else {
                Ok(format!(
                    "('{}', {}, '{}')",
                    s_borrow.name,
                    s_borrow.value,
                    s_borrow.comment.as_ref().unwrap()
                ))
            }
        }
    }
}

// Read header to a Python dictionary.
#[pyfunction]
#[pyo3(name = "read_header", signature = (path))]
fn _read_header(py: Python<'_>, path: &str) -> PyResult<Py<PyDict>> {
    let header = read_header(path).map_err(|e| PyIOError::new_err(format!("{}", e)))?;

    let dict = PyDict::new(py);

    for (key, value, comment) in header.into_iter() {
        let comment_any = if let Some(c) = comment {
            c.into_py_any(py)?
        } else {
            py.None()
        };

        match value {
            crate::header::FITSValue::String(s) => {
                dict.set_item(key, (s, comment_any))?;
            }
            crate::header::FITSValue::Integer(i) => {
                dict.set_item(key, (i, comment_any))?;
            }
            crate::header::FITSValue::Float(f) => {
                dict.set_item(key, (f, comment_any))?;
            }
            crate::header::FITSValue::Bool(b) => {
                dict.set_item(key, (b, comment_any))?;
            }
            crate::header::FITSValue::Null => {
                dict.set_item(key, (py.None(), comment_any))?;
            }
            crate::header::FITSValue::Invalid => {
                dict.set_item(key, (py.None(), comment_any))?;
            }
        };
    }

    return Ok(dict.into());
}

// Read header and convert to Header class.
#[pyfunction(name="read_header_to_class", signature = (path))]
fn _read_header_to_class(py: Python<'_>, path: &str) -> PyResult<Header> {
    let keywords_dict = _read_header(py, path)?;

    Python::attach(|py| {
        let py_header = Header {
            keywords: PyDict::new(py).into(),
        };

        for (name, value) in keywords_dict.into_bound(py).iter() {
            let kw_value = value.get_item(0)?;
            let kw_comment = value.get_item(1)?;

            let keyword = Py::new(
                py,
                Keyword {
                    name: name.extract::<String>()?,
                    value: kw_value.extract::<Py<PyAny>>()?,
                    comment: kw_comment.extract::<Option<Py<PyAny>>>()?,
                },
            )?;

            py_header.keywords.bind(py).set_item(name, keyword)?;
        }

        Ok(py_header)
    })
}

// Define the Python module
#[pymodule(name = "_rheader")]
fn rheader_python_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_read_header, m)?)?;
    m.add_function(wrap_pyfunction!(_read_header_to_class, m)?)?;
    m.add_class::<Header>()?;
    m.add_class::<Keyword>()?;
    Ok(())
}
