use crate::exceptions::PyValueError;
use crate::ffi;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::types::string::PyStringMethods;
use crate::types::PyFloat;
use crate::types::PyLong;
use crate::types::PyType;
use crate::{Bound, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};
use std::os::raw::c_char;

use num_rational::Ratio;
use num_rational::Rational32;

// To allow a new type, we need to do the following
// impl FromPyObject
// impl ToPyObject
// impl IntoPy
// Test

impl<'py> FromPyObject<'py> for Rational32 {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let py = obj.py();
        let py_numerator = unsafe {
            ffi::PyObject_GetAttrString(obj.as_ptr(), "numerator\0".as_ptr() as *const c_char)
        };
        // Opt 1
        // let num_owned: Py<PyLong> = unsafe { Py::from_owned_ptr_or_err(py, py_numerator)? };

        //Opt2
        // this already guarantees its converting to int?
        // if float then what?
        // check when thats the case what to do
        // let num_owned: Py<PyLong> =
        //     unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(py_numerator))? };
        //
        // let num_owned: Py<PyFloat> =
        //     unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Float(py_numerator))? };
        // let a = num_owned.bind(py);

        let num_owned: Py<PyLong> =
            unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Long(py_numerator))? };
        let num_owned = num_owned.bind(py);
        //  Convert num_owned to integer
        // let rs_numerator: i32 = num_owned.extract(py)?;
        dbg!(num_owned);
        // dbg!(a);
        // let rs_numerator: i128 = num_owned.extract(py)?;
        // dbg!(rs_numerator);
        // let ratio = Ratio::new(rs_numerator, 10i32);
        // dbg!(ratio);
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
                // "import fractions\npy_frac = fractions.Fraction(\"7.4\")",
                // "import fractions\npy_frac = fractions.Fraction(\"1.1\")",
                "import fractions\npy_frac = fractions.Fraction(1.1)",
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("py_frac").unwrap().unwrap();
            let roundtripped: Result<Rational32, PyErr> = py_frac.extract();
            // dbg!(roundtripped);
            // let roundtripped: Rational32 = py_frac.extract();
        })
    }
}
