/// Events from the ESP
#[derive(Debug, PartialEq, Eq)]
pub enum EspEvent {
    /// A new line has been received
    Line(String),

    /// A stream indicator has been received
    Stream(EspStream),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EspStream {
    Unknown,
    Result,
    Output,
    Exception,
    Log,
    SetPrompt,
    StateVec,
    Status,
    Prompt,
}

impl From<u8> for EspStream {
    fn from(byte: u8) -> EspStream {
        match byte {
            0o000 => EspStream::Unknown,
            0o201 => EspStream::Result,
            0o202 => EspStream::Output,
            0o203 => EspStream::Exception,
            0o204 => EspStream::Log,
            0o205 => EspStream::SetPrompt,
            0o206 => EspStream::StateVec,
            0o207 => EspStream::Status,
            0o200 => EspStream::Prompt,
            _ => EspStream::Unknown,
        }
    }
}

impl std::fmt::Display for EspStream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
