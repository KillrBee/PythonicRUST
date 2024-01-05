// Import necessary crates and modules
#[macro_use]
extern crate lazy_static;
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyInt};
use std::sync::{Arc, Mutex};

// Define a type alias for a tuple representing a value and its usage indicator
type Value = i32;
type UsageIndicator = bool;

// Define a global, thread-safe container to hold the array
lazy_static! {
    static ref ARRAY: Arc<Mutex<Vec<(Value, UsageIndicator)>>> = Arc::new(Mutex::new(Vec::new()));
}

// Function to allocate a value in the array, reusing an unused slot if available
fn allocate_value(value: Value) -> usize {
    let mut array = ARRAY.lock().unwrap();
    
    // Look for an unused slot
    for (index, &mut (ref val, ref mut used)) in array.iter_mut().enumerate() {
        if !*used {
            *val = value;  // Set the value
            *used = true;  // Mark the slot as used
            return index;  // Return the index of the slot
        }
    }

    // No unused slot found, so append a new one
    array.push((value, true));
    array.len() - 1  // Return the index of the new slot
}

// Function to deallocate a value, marking its slot as unused
fn deallocate_value(index: usize) {
    let mut array = ARRAY.lock().unwrap();
    if let Some(slot) = array.get_mut(index) {
        slot.1 = false;  // Mark the slot as unused
    }
}

// Function to handle PyObject and allocate a slot for its value in the array
fn handle_pyobject(obj: &PyAny) -> PyResult<usize> {
    if obj.is_instance::<PyString>()? {
        let string_value: &str = obj.extract()?;
        Ok(allocate_value(string_value.len() as i32))  // Example: use string length as value
    } else if obj.is_instance::<PyInt>()? {
        let int_value: i32 = obj.extract()?;
        Ok(allocate_value(int_value))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Unsupported type"))
    }
}

fn main() -> PyResult<()> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let py_string = PyString::new(py, "Hello, world");
    let py_int = PyInt::new(py, 42);

    // Handle PyObjects and allocate slots for their values in the array
    let string_index = handle_pyobject(py_string.as_ref(py))?;
    let int_index = handle_pyobject(py_int.as_ref(py))?;
    
    println!("String value allocated at index {}", string_index);
    println!("Integer value allocated at index {}", int_index);
    
    // Deallocate the values when done
    deallocate_value(string_index);
    deallocate_value(int_index);

    Ok(())
}
