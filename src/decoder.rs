use crate::debug_buffer;
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
            current_stream: EspStream::Log, // TODO check that Log is appropriate default
            pending_event: None,
        }
    }

    pub fn get_current_stream(&mut self) -> EspStream {
        self.current_stream
    }

    /// If any, returns next EspEvent, which is composed from internal buffer and given source data;
    /// otherwise, None if no enough data yet to compose such next event.
    pub fn decode(
        &mut self,
        src: &mut BytesMut,
        debug: bool,
    ) -> Result<Option<EspEvent>, EspError> {
        if let Some(event) = self.pending_event.take() {
            self.pending_event = None;
            if let EspEvent::Stream(s) = event {
                self.current_stream = s;
            }
            return Ok(Some(event));
        }

        if src.len() == 0 {
            // no more data, so nothing new completed of course:
            return Ok(None);
        }

        let mut k = 0;
        let max_buffer_length = self.max_buffer_length;

        loop {
            if k >= src.len() {
                return Ok(None);
            }

            let byte = src[k];
            match byte {
                b'\n' => {
                    let buffer = mem::replace(&mut self.buffer, Vec::new());
                    if debug {
                        debug_buffer("RCVD", &buffer, true);
                    }

                    let _ = src.split_to(k + 1);
                    if buffer.len() > 0 {
                        // stream indicator?
                        let pre_byte = buffer[buffer.len() - 1];
                        if 0o200 <= pre_byte && pre_byte <= 0o207 {
                            let new_stream = EspStream::from(pre_byte);
                            let stream_event = Some(EspEvent::Stream(new_stream));
                            // non-empty contents before stream indicator?
                            if buffer.len() > 1 {
                                // "queue" this stream event:
                                self.pending_event = stream_event;
                                // and complete currently buffered stuff (modulo stream indicator):
                                let result = String::from_utf8_lossy(&buffer[0..buffer.len() - 1]);
                                return Ok(Some(EspEvent::Line(result.to_string())));
                            } else {
                                self.current_stream = new_stream;
                                return Ok(stream_event);
                            }
                        } else {
                            let result = String::from_utf8_lossy(&buffer[..]);
                            return Ok(Some(EspEvent::Line(result.to_string())));
                        }
                    }
                }

                _ if self.buffer.len() < max_buffer_length => {
                    self.buffer.push(byte);
                }

                _ => {
                    println!("WARN: buffer full, ignoring byte={:?}", byte);
                }
            }
            k += 1;
        }
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
            match dec.decode(bytes, false) {
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
