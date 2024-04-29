use crate::err::PyErr;
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

static FRACTION_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_fraction_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    FRACTION_CLS.get_or_try_init_type_ref(py, "fractions", "Fraction")
}

macro_rules! rational_conversion {
    ($int: ty) => {
        impl<'py> FromPyObject<'py> for Ratio<$int> {
            fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
                // check if its decimal
                // only make it work if feature=decimal is enabled
                // otherwise return error
                let py = obj.py();
                let py_numerator = unsafe {
                    ffi::PyObject_GetAttrString(
                        obj.as_ptr(),
                        "numerator\0".as_ptr() as *const c_char,
                    )
                };
                let py_denominator = unsafe {
                    ffi::PyObject_GetAttrString(
                        obj.as_ptr(),
                        "denominator\0".as_ptr() as *const c_char,
                    )
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
                let rs_numerator: $int = numerator_owned.extract()?;
                let rs_denominator: $int = denominator_owned.extract()?;
                Ok(Ratio::new(rs_numerator, rs_denominator))
            }
        }

        impl ToPyObject for Ratio<$int> {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                // TODO: handle error gracefully when ToPyObject can error
                // look up the decimal.Decimal
                // PyErr { type: <class 'ModuleNotFoundError'>, value: ModuleNotFoundError("No module named 'fractionxs'"), traceback: None }
                // let func_res = || -> PyResult<PyObject> {
                let func_res = || -> PyResult<PyObject> {
                    // let frac_cls = get_fraction_cls(py).expect("failed to load fractions.Fraction");
                    let frac_cls = get_fraction_cls(py)?;

                    // expect("failed to load fractions.Fraction");

                    // now call the constructor with the Rust Decimal string-ified
                    // to not be lossy
                    // TODO: is this waht we want? We want it in either
                    // string and ints, so choose what to do?
                    let ret = frac_cls.call1((self.to_string(),))?;
                    // can this fail?
                    Ok(ret.to_object(py))
                    // Ok(())
                };
                match func_res() {
                    Ok(obj) => obj,
                    //TODO: Is this the way to handle this?
                    Err(e) => panic!("{:?}", e),
                }
            }
        }
        impl IntoPy<PyObject> for Ratio<$int> {
            fn into_py(self, py: Python<'_>) -> PyObject {
                self.to_object(py)
            }
        }
    };
}

rational_conversion!(i32);
// rational_conversion!(i64);
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
            // TODO: Add test to make sure "10" and 10.4" are handled correctly
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
            // let rs_frac = Ratio::new(10, 1);
            let rs_frac = Ratio::new(10, 1);
            assert_eq!(roundtripped, rs_frac);

            // dbg!(roundtripped);
            // let roundtripped: Rational32 = py_frac.extract();
        })
    }

    #[test]
    fn test_roundtrip() {
        Python::with_gil(|py| {
            let rs_frac = Ratio::new(10i32, 3i32);
            // let rs_frac = Ratio::from_float(0.5).unwrap();
            // rational32

            // let num = Ratio::from_float(0.5).unwrap();
            // let den = Ratio::from_float(1.0).unwrap();
            let py_frac = rs_frac.into_py(py);
            let roundtripped: Rational32 = py_frac.extract(py).unwrap();
            assert_eq!(roundtripped, rs_frac);
        })
    }
}

// Fraction("10.3") and Fraction(10.3) are different in PYthon
// Handle those
// Handle BigInt on to_object when you do Ratio::from_float(0.5).unwrap();
// that is equivalent to pythons Fraction(0.5)
