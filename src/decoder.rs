use crate::error::*;
use crate::event::*;

use bytes::BytesMut;
use std::mem;

pub struct EspDecoder {
    max_buffer_length: usize,
    buffer: Vec<u8>,
    current_stream: EspStream,
    pending_event: Option<EspEvent>,
}

impl EspDecoder {
    pub fn new(max_buffer_length: usize) -> EspDecoder {
        EspDecoder {
            max_buffer_length,
            buffer: Vec::new(),
            current_stream: EspStream::Unknown,
            pending_event: None,
        }
    }

    /// If any, returns next EspEvent, which is composed from internal buffer and given source data;
    /// otherwise, None if no enough data yet to compose such next event.
    pub fn decode(&mut self, src: &mut BytesMut) -> Result<Option<EspEvent>, EspError> {
        if let Some(event) = self.pending_event.take() {
            self.pending_event = None;
            return Ok(Some(event));
        }

        if src.len() == 0 {
            // no more data, so nothing new completed of course:
            return Ok(None);
        }

        let mut k = 0;
        let mut buffer_len = self.buffer.len();
        let max_buffer_length = self.max_buffer_length;

        loop {
            if k >= src.len() {
                return Ok(None);
            }

            let byte = src[k];
            match byte {
                // line completed?
                b'\n' => {
                    let _ = src.split_to(k + 1);
                    // do not notify empty lines (at least for now):
                    if buffer_len > 0 {
                        return Ok(Some(self.complete_line()));
                    }
                }

                // stream indicator?
                _ if 0o200 <= byte && byte <= 0o207 => {
                    let _ = src.split_to(k + 1);
                    let stream_event = Some(EspEvent::Stream(EspStream::from(byte)));
                    if buffer_len > 0 {
                        // "queue" this stream event:
                        self.pending_event = stream_event;
                        // and complete currently buffered stuff:
                        let event = Some(self.complete_line());
                        self.current_stream = EspStream::from(byte);
                        return Ok(event);
                    } else {
                        return Ok(stream_event);
                    }
                }

                _ if buffer_len < max_buffer_length => {
                    self.buffer.push(byte);
                    buffer_len += 1;
                }

                _ => {
                    println!("WARN: buffer full, ignoring byte={:?}", byte);
                }
            }
            k += 1;
        }
    }

    fn complete_line(&mut self) -> EspEvent {
        let buffer = mem::replace(&mut self.buffer, Vec::new());
        let result = format!("{}", String::from_utf8_lossy(&buffer[..]));
        EspEvent::Line(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::decoder;
    use crate::error::*;
    use crate::event::*;
    use bytes::BytesMut;

    fn consume(
        dec: &mut decoder::EspDecoder,
        bytes: &mut BytesMut,
    ) -> Vec<Result<Option<EspEvent>, EspError>> {
        let mut result = Vec::new();
        loop {
            match dec.decode(bytes) {
                Ok(None) => {
                    break;
                }
                output => result.push(output),
            }
        }
        return result;
    }

    #[test]
    fn unfinished_line() {
        let mut dec = decoder::EspDecoder::new(4096);
        let mut bytes = BytesMut::from(&*b"unfinished line".to_vec());
        let result = consume(&mut dec, &mut bytes);

        assert_eq!(result, vec![],);
    }
}
