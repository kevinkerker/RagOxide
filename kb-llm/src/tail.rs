//! The last bytes of a pipe of the server, kept in memory only (F-F4).

use std::collections::VecDeque;

/// How many bytes of each pipe are kept.
pub(crate) const TAIL_BYTES: usize = 64 * 1024;

/// A ring of the last [`TAIL_BYTES`] bytes written to a pipe.
#[derive(Debug, Default)]
pub(crate) struct Tail {
    bytes: VecDeque<u8>,
}

impl Tail {
    /// Appends `chunk`, dropping the oldest bytes beyond the limit.
    pub(crate) fn push(&mut self, chunk: &[u8]) {
        let chunk = &chunk[chunk.len().saturating_sub(TAIL_BYTES)..];
        let overflow = (self.bytes.len() + chunk.len()).saturating_sub(TAIL_BYTES);
        self.bytes.drain(..overflow);
        self.bytes.extend(chunk);
    }

    /// The last complete lines within the last `max` bytes, as lossy UTF-8.
    pub(crate) fn last_lines(&self, max: usize) -> String {
        let skip = self.bytes.len().saturating_sub(max);
        let mut bytes: Vec<u8> = self.bytes.iter().skip(skip).copied().collect();
        if skip > 0
            && let Some(newline) = bytes.iter().position(|b| *b == b'\n')
        {
            bytes.drain(..=newline);
        }
        String::from_utf8_lossy(&bytes).trim_end().to_owned()
    }

    /// Number of bytes kept.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_last_bytes() {
        let mut tail = Tail::default();
        for _ in 0..100 {
            tail.push(&[b'x'; 10_000]);
        }
        assert_eq!(tail.len(), TAIL_BYTES);
        tail.push(&vec![b'y'; 3 * TAIL_BYTES]);
        assert_eq!(tail.len(), TAIL_BYTES);
        assert!(tail.last_lines(10).bytes().all(|b| b == b'y'));
    }

    #[test]
    fn last_lines_starts_at_a_line_boundary() {
        let mut tail = Tail::default();
        tail.push(b"first line\nsecond line\nthird line\n");
        assert_eq!(tail.last_lines(1000), "first line\nsecond line\nthird line");
        assert_eq!(tail.last_lines(20), "third line");
    }
}
