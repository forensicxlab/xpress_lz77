mod xpress_lz77_huffman;
mod xpress_lz77_plain;

use pyo3::prelude::*;
use xpress_lz77_huffman::lz77_huffman_decompress;
use xpress_lz77_plain::lz77_plain_decompress;

#[pyfunction]
fn lz77_plain_decompress_py(in_buf: &[u8]) -> PyResult<Vec<u8>> {
    lz77_plain_decompress(in_buf)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pyfunction]
fn lz77_huffman_decompress_py(in_buf: &[u8], output_size: usize) -> PyResult<Vec<u8>> {
    lz77_huffman_decompress(in_buf, output_size)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pymodule]
fn xpress_lz77(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(lz77_plain_decompress_py, m)?)?;
    m.add_function(wrap_pyfunction!(lz77_huffman_decompress_py, m)?)?;
    Ok(())
}
