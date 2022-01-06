/// Encodes a line so it can be sent to the ESP.
/// The given string may include or omit the trailing '\n'.
/// The encoding is simply the insertion of a '\0' character before the '\n'.
pub fn encode_line(line: &str) -> Vec<u8> {
    let bytes = line.as_bytes();
    assert!(!bytes.contains(&b'\0'));

    let len = bytes.len();

    // +1 for inserted '\0' right before the '\n'
    let mut encoded_size = len + 1;

    // len excluding any trailing '\n'
    let mut len_no_ln = len;

    if len > 0 && bytes[len - 1] == b'\n' {
        len_no_ln -= 1;
    } else {
        // + 1 for added trailing '\n'
        encoded_size += 1;
    }

    let mut encoded = vec![0u8; encoded_size];
    encoded[..len_no_ln].clone_from_slice(&bytes[..len_no_ln]);
    encoded[encoded_size - 2] = b'\0';
    encoded[encoded_size - 1] = b'\n';

    encoded
}

#[cfg(test)]
mod tests {
    use crate::encoder;

    #[test]
    fn encode_line_tests() {
        assert_eq!(encoder::encode_line("Foo"), b"Foo\0\n");
        assert_eq!(encoder::encode_line("Foo\n"), b"Foo\0\n");

        assert_eq!(encoder::encode_line(""), b"\0\n");
    }
}
