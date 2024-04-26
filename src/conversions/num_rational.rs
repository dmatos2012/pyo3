use crate::exceptions::PyValueError;
use crate::ffi;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::types::string::PyStringMethods;
use crate::types::PyType;
use crate::{Bound, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};

use num_rational::Rational32;

// To allow a new type, we need to do the following
// impl FromPyObject
// impl ToPyObject
// impl IntoPy
// Test

impl<'py> FromPyObject<'py> for Rational32 {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        // let x = obj.extract();
        let py = obj.py();
        let pyobj: PyObject = obj.extract().unwrap();
        let num_owned =
            unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(obj.as_ptr()))? };
        // let x: PyObject = obj.py().unwrap();
        // let x = obj.py();
        dbg!(pyobj);
        Ok(Rational32::from(5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::err::PyErr;
    use crate::types::dict::PyDictMethods;
    use crate::types::PyDict;
    #[test]
    fn handle_fraction() {
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            py.run_bound(
                "import fractions\npy_frac = fractions.Fraction(\"7\")",
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("py_frac").unwrap().unwrap();
            let roundtripped: Result<Rational32, PyErr> = py_frac.extract();
            dbg!(roundtripped);
            // let roundtripped: Rational32 = py_frac.extract();
        })
    }
}
