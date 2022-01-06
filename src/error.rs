#[derive(Debug)]
pub enum EspError {
    Io(std::io::Error),
}

impl From<std::io::Error> for EspError {
    fn from(err: std::io::Error) -> EspError {
        EspError::Io(err)
    }
}

impl PartialEq for EspError {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (EspError::Io(a), EspError::Io(b)) => a.kind() == b.kind(),
        }
    }
}

// the following based on https://blog.burntsushi.net/rust-error-handling/#the-error-trait
// plus some adjustments as hinted by the compiler

impl std::fmt::Display for EspError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            EspError::Io(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for EspError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            EspError::Io(ref err) => Some(err),
        }
    }
}
