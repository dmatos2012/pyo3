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
use num_rational::{Rational32, Rational64};

// To allow a new type, we need to do the following
// impl FromPyObject
// impl ToPyObject
// impl IntoPy
// Test

impl<'py> FromPyObject<'py> for Rational32 {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        // check if its decimal
        // only make it work if feature=decimal is enabled
        // otherwise return error
        let py = obj.py();
        let py_numerator = unsafe {
            ffi::PyObject_GetAttrString(obj.as_ptr(), "numerator\0".as_ptr() as *const c_char)
        };
        let py_denominator = unsafe {
            ffi::PyObject_GetAttrString(obj.as_ptr(), "denominator\0".as_ptr() as *const c_char)
        };
        // Opt 1
        // let num_owned: Py<PyLong> = unsafe { Py::from_owned_ptr_or_err(py, py_numerator)? };

        //Opt2
        // this already guarantees its converting to int?
        // if float then what?
        // check when thats the case what to do
        // let numerator_owned: Py<PyLong> =
        //     unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Index(py_numerator))? };
        //
        // let numerator_owned: Py<PyFloat> =
        //     unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Float(py_numerator))? };
        // let a = numerator_owned.bind(py);

        let numerator_owned: Py<PyLong> =
            unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Long(py_numerator))? };
        let numerator_owned = numerator_owned.bind(py);
        let denominator_owned: Py<PyLong> =
            unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Long(py_denominator))? };
        let denominator_owned = denominator_owned.bind(py);

        //  Convert numerator_owned to integer
        // TODO????
        // Wont this cause overflow?
        let rs_numerator: i32 = numerator_owned.extract()?;
        let rs_denominator: i32 = denominator_owned.extract()?;
        Ok(Ratio::new(rs_numerator, rs_denominator))
    }
}

fn get_fraction_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    FRACTION_CLS.get_or_try_init_type_ref(py, "fractionxs", "Fraction")
}

static FRACTION_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
impl ToPyObject for Rational32 {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        // TODO: handle error gracefully when ToPyObject can error
        // look up the decimal.Decimal
        // PyErr { type: <class 'ModuleNotFoundError'>, value: ModuleNotFoundError("No module named 'fractionxs'"), traceback: None }

        let frac_cls = get_fraction_cls(py).expect("failed to load fractions.Fraction");

        // now call the constructor with the Rust Decimal string-ified
        // to not be lossy
        // TODO: is this waht we want? We want it in either
        // string and ints, so choose what to do?
        let ret = frac_cls
            .call1((self.to_string(),))
            .expect("failed to call fractions.Fraction(value)");
        ret.to_object(py)
    }
}
impl IntoPy<PyObject> for Rational32 {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

// You still need to implement
// num.into_py(py)
// to convert rs obj to py obj

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
                "import fractions\npy_frac = fractions.Fraction(10)",
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("py_frac").unwrap().unwrap();
            // let roundtripped: Rational32 = py_frac.extract().unwrap();
            let roundtripped: Rational32 = py_frac.extract().unwrap();
            let rs_frac = Ratio::new(10, 1);
            assert_eq!(roundtripped, rs_frac);

            // dbg!(roundtripped);
            // let roundtripped: Rational32 = py_frac.extract();
        })
    }

    #[test]
    fn test_roundtrip() {
        Python::with_gil(|py| {
            let rs_frac = Ratio::new(10i32, 1i32);
            let py_frac = rs_frac.into_py(py);
            let roundtripped: Rational32 = py_frac.extract(py).unwrap();
            assert_eq!(roundtripped, rs_frac);
        })
    }
}
