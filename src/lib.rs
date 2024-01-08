use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicI64;
use std::thread::Thread;
use pyo3::ffi::{hashfunc, PyObject_Hash};

use pyo3::prelude::*;
use pyo3::types::{PyCode, PyTuple};

#[pyfunction]
fn init_monitor(tool_id: u8) -> PyResult<Py<PyMonitor>> {
    Python::with_gil(|py| {
        if tool_id > 5 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("tool_id must be between 0 and 5"));
        }
        let monitoring = py.import("sys")?.getattr("monitoring")?;
        let use_tool_id = monitoring.getattr("use_tool_id")?;
        let register_callback = monitoring.getattr("register_callback")?;
        let set_events = monitoring.getattr("set_events")?;

        let events = monitoring.getattr("events")?;
        let call = events.getattr("PY_START")?;
        let c_return = events.getattr("PY_YIELD")?;
        let c_raise = events.getattr("PY_RAISE")?;

        let monitor_obj = build_monitor(tool_id);
        let tool_id_args = PyTuple::new(py, [tool_id.into_py(py), "rs_py_perf".into_py(py)]);
        use_tool_id.call(tool_id_args, None)?;

        let cb: Py<PyAny> = monitor_obj.getattr(py, "__call__")?.into();

        set_events.call(PyTuple::new(py, [tool_id.into_py(py), call.clone().into()]), None)?;
        let args = PyTuple::new(py, [tool_id.into_py(py), call.clone().into(), cb.clone()]);
        register_callback.call(args, None)?;

        set_events.call(PyTuple::new(py, [tool_id.into_py(py), c_return.clone().into()]), None)?;
        let args = PyTuple::new(py, [tool_id.into_py(py), c_return.clone().into(), cb.clone()]);
        register_callback.call(args, None)?;

        set_events.call(PyTuple::new(py, [tool_id.into_py(py), c_raise.clone().into()]), None)?;
        let args = PyTuple::new(py, [tool_id.into_py(py), c_raise.clone().into(), cb.clone()]);
        register_callback.call(args, None)?;
        Ok(monitor_obj)
    })
}

fn build_monitor(id: u8) -> Py<PyMonitor> {
    Python::with_gil(|py| {
        Py::new(py, PyMonitor::new(id)).unwrap()
    })
}

pub enum Event {
    Call,
    Return,
    Raise,
}

#[pyclass]
pub struct PyMonitor {
    id: u8,
    callables: HashMap<isize, i64>,
}

#[pymethods]
impl PyMonitor {
    #[new]
    fn new(id: u8) -> Self {
        PyMonitor {
            id,
            callables: HashMap::new(),
        }
    }

    #[pyo3(signature = (code, instruction_offset, callable, args0))]
    fn __call__(
        &mut self,
        // py: &Python<'_>,
        code: &PyCode,
        instruction_offset: i64,
        callable: PyObject,
        args0: Py<PyAny>,
    ) -> PyResult<()> {
        self.callables.entry(key)
            .or_insert(AtomicI64::new(0)).fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}


/// Formats the sum of two numbers as string.
///
/// func(code: CodeType, instruction_offset: int, callable: object, arg0: object | MISSING) -> DISABLE | Any
///

/// A Python module implemented in Rust.
#[pymodule]
fn rs_py_perf(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMonitor>()?;
    m.add_function(wrap_pyfunction!(init_monitor, m)?)?;
    Ok(())
}
