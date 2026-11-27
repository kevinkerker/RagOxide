//! Server-sent events: splits the bytes of a response into the payloads of its events.
//!
//! Lines are cut at `\n` (a `\r` before it is dropped) on the raw bytes, so a piece of
//! the response may end anywhere, also inside a UTF-8 sequence; only complete lines are
//! decoded. `data:` lines are joined with `\n`, an empty line ends the event, lines
//! starting with `:` are comments. `error:` lines (older servers send errors that way)
//! are delivered as error payloads. Other fields are ignored.

/// A complete event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SseEvent {
    /// The joined `data:` lines.
    Data(String),
    /// The text of an `error:` line.
    Error(String),
}

/// Error of the parser: a line that is not UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NotUtf8;

/// Incremental parser.
#[derive(Debug, Default)]
pub(crate) struct SseParser {
    line: Vec<u8>,
    data: Option<String>,
}

impl SseParser {
    /// Feeds the next bytes and appends the events they complete to `out`.
    pub(crate) fn push(&mut self, bytes: &[u8], out: &mut Vec<SseEvent>) -> Result<(), NotUtf8> {
        let mut rest = bytes;
        while let Some(newline) = rest.iter().position(|b| *b == b'\n') {
            self.line.extend_from_slice(&rest[..newline]);
            rest = &rest[newline + 1..];
            let mut line = std::mem::take(&mut self.line);
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            let line = String::from_utf8(line).map_err(|_| NotUtf8)?;
            self.line_complete(&line, out);
        }
        self.line.extend_from_slice(rest);
        Ok(())
    }

    /// Whether bytes of an unfinished line or event are left.
    pub(crate) fn has_partial(&self) -> bool {
        !self.line.is_empty() || self.data.is_some()
    }

    fn line_complete(&mut self, line: &str, out: &mut Vec<SseEvent>) {
        if line.is_empty() {
            if let Some(data) = self.data.take() {
                out.push(SseEvent::Data(data));
            }
            return;
        }
        if line.starts_with(':') {
            return;
        }
        let (field, value) = match line.split_once(':') {
            Some((field, value)) => (field, value.strip_prefix(' ').unwrap_or(value)),
            None => (line, ""),
        };
        match field {
            "data" => match &mut self.data {
                Some(data) => {
                    data.push('\n');
                    data.push_str(value);
                }
                None => self.data = Some(value.to_owned()),
            },
            "error" => out.push(SseEvent::Error(value.to_owned())),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_in_pieces(bytes: &[u8], size: usize) -> Vec<SseEvent> {
        let mut parser = SseParser::default();
        let mut out = Vec::new();
        for piece in bytes.chunks(size) {
            parser.push(piece, &mut out).unwrap();
        }
        assert!(!parser.has_partial());
        out
    }

    #[test]
    fn events_survive_any_split() {
        let text =
            "data: {\"a\":\"Grüße 😀\"}\r\n\r\n: comment\n\ndata: x\ndata: y\n\ndata: [DONE]\n\n";
        let expected = vec![
            SseEvent::Data("{\"a\":\"Grüße 😀\"}".into()),
            SseEvent::Data("x\ny".into()),
            SseEvent::Data("[DONE]".into()),
        ];
        for size in 1..text.len() {
            assert_eq!(
                parse_in_pieces(text.as_bytes(), size),
                expected,
                "size {size}"
            );
        }
    }

    #[test]
    fn partial_line_is_reported() {
        let mut parser = SseParser::default();
        let mut out = Vec::new();
        parser.push(b"data: {\"cho", &mut out).unwrap();
        assert!(out.is_empty());
        assert!(parser.has_partial());
        let mut parser = SseParser::default();
        parser.push(b"data: x\n", &mut out).unwrap();
        assert!(parser.has_partial(), "event without its empty line");
    }

    #[test]
    fn error_lines_and_invalid_utf8() {
        let mut parser = SseParser::default();
        let mut out = Vec::new();
        parser.push(b"error: {\"code\":500}\n", &mut out).unwrap();
        assert_eq!(out, vec![SseEvent::Error("{\"code\":500}".into())]);
        assert_eq!(parser.push(b"data: \xff\xfe\n", &mut out), Err(NotUtf8));
    }
}
