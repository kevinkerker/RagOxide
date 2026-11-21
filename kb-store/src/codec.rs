//! Conversions between the contract's values and what SQLite stores.

use std::path::Path;

use kb_core::{Anchor, Chat, Document, Error, Result, Timestamp, path_key};

/// Sign bit of a 64-bit integer.
const SIGN: u64 = 1 << 63;

/// Encodes a [`Timestamp`] as SQLite `INTEGER` (signed 64 bit) so that the order of the
/// integers is the order of the timestamps: the sign bit is flipped, so 0 becomes
/// `i64::MIN` and `u64::MAX` becomes `i64::MAX`. Every `u64` fits, also above
/// `i64::MAX`.
pub(crate) fn encode_time(time: Timestamp) -> i64 {
    i64::from_ne_bytes((time.0 ^ SIGN).to_ne_bytes())
}

/// Inverse of [`encode_time`].
pub(crate) fn decode_time(value: i64) -> Timestamp {
    Timestamp(u64::from_ne_bytes(value.to_ne_bytes()) ^ SIGN)
}

/// Identity key of a record's path: [`path_key`], or for a path that is not absolute
/// (which the contract never produces) its lower-cased text, as `MockStore` does.
pub(crate) fn record_key(path: &Path) -> String {
    path_key(path).unwrap_or_else(|_| path.to_string_lossy().to_lowercase())
}

fn finite_anchors(anchors: &[Anchor]) -> bool {
    anchors.iter().all(|anchor| match anchor {
        Anchor::Page { bbox, .. } => [bbox.left, bbox.top, bbox.right, bbox.bottom]
            .iter()
            .all(|v| v.is_finite()),
        Anchor::Slide { .. } | Anchor::Sheet { .. } | Anchor::Paragraph { .. } => true,
    })
}

fn not_finite(what: &str) -> Error {
    Error::InvalidInput(format!(
        "{what} contains a number that is not finite (NaN or infinity), which cannot be stored"
    ))
}

/// Refuses a document with a page size or box that is NaN or infinite: JSON has no
/// such numbers, the value would be written as `null` and could not be read back.
pub(crate) fn check_document(document: &Document) -> Result<()> {
    let pages = document
        .pages
        .iter()
        .all(|p| p.width.is_finite() && p.height.is_finite());
    let elements = document.elements.iter().all(|element| {
        finite_anchors(&element.anchors)
            && match &element.kind {
                kb_core::ElementKind::Table(table) => table.cells.iter().all(|cell| {
                    cell.anchor
                        .as_ref()
                        .is_none_or(|a| finite_anchors(std::slice::from_ref(a)))
                }),
                _ => true,
            }
    });
    if pages && elements {
        Ok(())
    } else {
        Err(not_finite(&format!(
            "document {}",
            document.source.document_id
        )))
    }
}

/// Like [`check_document`] for the boxes in the citations of a chat.
pub(crate) fn check_chat(chat: &Chat) -> Result<()> {
    let finite = chat.turns.iter().all(|turn| {
        turn.answer
            .citations
            .iter()
            .all(|c| finite_anchors(&c.chunk.provenance.anchors))
    });
    if finite {
        Ok(())
    } else {
        Err(not_finite(&format!("chat {}", chat.id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_encoding_keeps_order_and_value() {
        let values = [
            0,
            1,
            1_794_700_800_000,
            i64::MAX as u64,
            SIGN,
            u64::MAX - 1,
            u64::MAX,
        ];
        for pair in values.windows(2) {
            let (a, b) = (Timestamp(pair[0]), Timestamp(pair[1]));
            assert!(encode_time(a) < encode_time(b), "{a:?} < {b:?}");
        }
        for v in values {
            assert_eq!(decode_time(encode_time(Timestamp(v))), Timestamp(v));
        }
    }
}
