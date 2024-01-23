use pyo3::prelude::*;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing_subscriber::EnvFilter;

#[pyclass]
struct PyTcpStream {
    fd: u32,
    inner: Option<TcpStream>,
}

#[pymethods]
impl PyTcpStream {
    // fn send<'py>(&'py mut self, py: Python<'py>, content: Vec<u8>) -> PyResult<&PyAny> {
    //     let mut stream = self.inner.take().unwrap();
    //
    //     pyo3_asyncio::tokio::future_into_py(py, async move {
    //         let (mut r,mut w) = stream.split();
    //         w.write_all(&content).await.unwrap();
    //         r.readable().await.unwrap();
    //         // stream.shutdown().await.unwrap();
    //         Ok(Python::with_gil(|py| py.None()))
    //     })
    // }

    fn send<'py>(&'py mut self, py: Python<'py>, content: Vec<u8>) -> PyResult<&PyAny> {
        let mut stream = self.inner.take().unwrap();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let (mut r,mut w) = stream.split();
            w.write_all(&content).await.unwrap();
            timeout(std::time::Duration::from_secs(10), r.readable()).await.unwrap().unwrap();
            // stream.shutdown().await.unwrap();
            Ok(Python::with_gil(|py| py.None()))
        })
    }
}

#[pyfunction]
fn connect(py: Python<'_>, address: String, port: u32) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let stream = TcpStream::connect(format!("{}:{}", address, port))
            .await
            .unwrap();
        let fd = stream.as_raw_fd() as u32;
        Ok(PyTcpStream {
            fd,
            inner: Some(stream),
        })
    })
}

#[pymodule]
fn tcptest(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // console_subscriber::init();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .pretty()
        .init();

    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_class::<PyTcpStream>()?;
    Ok(())
}
