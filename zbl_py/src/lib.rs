use ::zbl::windows::Win32::Foundation::HWND;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use std::ffi::c_void;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("window with given name not found: {0}")]
    WindowNotFoundError(String),
    #[error("windows api error: {0}")]
    WindowsError(#[from] ::zbl::windows::core::Error),
    #[error("frame channel error")]
    FrameChannelError(#[from] std::sync::mpsc::RecvError),
    #[error("neither name nor handle is set")]
    NeitherNameNorHandleIsSet,
}

impl From<Error> for PyErr {
    fn from(error: Error) -> Self {
        PyRuntimeError::new_err(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[pyclass(unsendable)]
pub struct Frame {
    width: u32,
    height: u32,
    row_pitch: u32,
    ptr: *mut c_void,
}

#[pymethods]
impl Frame {
    #[getter]
    pub fn width(&self) -> usize {
        self.width as usize
    }

    #[getter]
    pub fn height(&self) -> usize {
        self.height as usize
    }

    #[getter]
    pub fn row_pitch(&self) -> usize {
        self.row_pitch as usize
    }

    #[getter]
    pub fn ptr(&self) -> usize {
        self.ptr as usize
    }
}

#[pyclass(unsendable)]
pub struct Capture {
    inner: ::zbl::Capture,
}

impl Capture {
    pub fn from_window(window: ::zbl::Window) -> Result<Self> {
        // TODO expose as python api, call on module loaded?
        ::zbl::init();
        let capture = ::zbl::Capture::new(window)?;
        Ok(Self { inner: capture })
    }

    pub fn from_window_name(name: &str) -> Result<Self> {
        let window = ::zbl::Window::find_first(name)
            .ok_or_else(|| Error::WindowNotFoundError(name.to_string()))?;
        Ok(Self::from_window(window)?)
    }

    fn _start(&self) -> Result<()> {
        Ok(self.inner.start()?)
    }

    fn _grab(&mut self) -> Result<Option<Frame>> {
        if let Some(::zbl::Frame { texture, ptr }) = self.inner.grab()? {
            Ok(Some(Frame {
                width: texture.desc.Width,
                height: texture.desc.Height,
                row_pitch: ptr.RowPitch,
                ptr: ptr.pData,
            }))
        } else {
            Ok(None)
        }
    }

    fn _stop(&mut self) -> Result<()> {
        Ok(self.inner.stop()?)
    }
}

#[pymethods]
impl Capture {
    #[new]
    pub fn new(name: Option<&str>, handle: Option<i32>) -> PyResult<Self> {
        if let Some(name) = name {
            Ok(Self::from_window_name(name)?)
        } else if let Some(handle) = handle {
            Ok(Self::from_window(::zbl::Window::new(HWND(
                handle as isize,
            )))?)
        } else {
            Err(Error::NeitherNameNorHandleIsSet)?
        }
    }

    #[getter]
    pub fn window(&self) -> PyResult<isize> {
        Ok(self.inner.window().handle.0)
    }

    #[getter]
    pub fn process_id(&self) -> PyResult<usize> {
        Ok(self.inner.window().get_process_id() as usize)
    }

    pub fn start(&self) -> PyResult<()> {
        Ok(self._start()?)
    }

    pub fn grab(&mut self) -> PyResult<Option<Frame>> {
        Ok(self._grab()?)
    }

    pub fn stop(&mut self) -> PyResult<()> {
        Ok(self._stop()?)
    }
}

#[pymodule]
#[pyo3(name = "zbl")]
fn zbl(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<Frame>()?;
    module.add_class::<Capture>()?;
    Ok(())
}
