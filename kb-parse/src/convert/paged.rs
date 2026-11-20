//! PDF and images: pages, boxes in the frame of the displayed page, `from_ocr`.

use std::collections::BTreeMap;

use kb_core::{Anchor, BoundingBox, Page};
use serde_json::Value;

use super::tree::DoclingJson;

/// What the conversion report (`ConversionResult.document.confidence`) says about the
/// origin of a page's text; not part of the docling JSON.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TextOrigin {
    /// The page has a text layer (`parse_score` set).
    pub text_layer: bool,
    /// The text layer is a scan's OCR layer (`scanned_text_layer`, ADR 0094, 0106).
    pub scanned_layer: bool,
    /// The pipeline recognised text on the page itself (`ocr_cells` not empty).
    pub recognised: bool,
}

impl TextOrigin {
    /// `Page::from_ocr` of a PDF page: text from a recognition wholly or in part, an
    /// adopted scan layer, or no text layer at all.
    pub(crate) fn is_recognised(&self) -> bool {
        self.scanned_layer || self.recognised || !self.text_layer
    }
}

/// One page as docling reports it, plus the factor to the contract's unit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PageFrame {
    /// Page height in docling's unit (points), for boxes with origin bottom left.
    pub height: f32,
    /// Factor from docling's unit to the contract's unit (1 for PDF, pixels per point for
    /// images).
    pub scale_x: f32,
    /// Vertical factor.
    pub scale_y: f32,
    /// Page width in the contract's unit (exact pixel count for images).
    pub out_width: f32,
    /// Page height in the contract's unit.
    pub out_height: f32,
}

impl PageFrame {
    /// The page in the contract's frame.
    pub(crate) fn page(&self, page: u32, from_ocr: bool) -> Page {
        Page {
            page,
            width: self.out_width,
            height: self.out_height,
            from_ocr,
        }
    }

    /// A docling box (`{l, t, r, b, coord_origin}`) in the contract's frame: origin top
    /// left of the displayed page, ordered edges, clamped to the page.
    pub(crate) fn bbox(&self, bbox: &Value) -> Option<BoundingBox> {
        let num = |k: &str| bbox.get(k).and_then(Value::as_f64).map(|v| v as f32);
        let (l, t, r, b) = (num("l")?, num("t")?, num("r")?, num("b")?);
        let (top, bottom) = match bbox.get("coord_origin").and_then(Value::as_str) {
            Some("BOTTOMLEFT") => (self.height - t, self.height - b),
            _ => (t, b),
        };
        let (w, h) = (self.out_width, self.out_height);
        let x = |v: f32| (v * self.scale_x).clamp(0.0, w);
        let y = |v: f32| (v * self.scale_y).clamp(0.0, h);
        let (left, right) = (x(l.min(r)), x(l.max(r)));
        let (top, bottom) = (y(top.min(bottom)), y(top.max(bottom)));
        Some(BoundingBox {
            left,
            top,
            right,
            bottom,
        })
    }
}

/// The pages of the conversion by number. For images, `pixels` gives the size of each
/// page image in pixels (EXIF orientation applied), which fixes the factor from points
/// to pixels; for PDF it is `None` and the factor is 1.
pub(crate) fn frames(json: &DoclingJson<'_>, images: bool) -> BTreeMap<u32, PageFrame> {
    let mut frames = BTreeMap::new();
    for (key, entry) in json.pages().into_iter().flatten() {
        let Ok(number) = key.parse::<u32>() else {
            continue;
        };
        let size = |v: Option<&Value>, k: &str| {
            v.and_then(|s| s.get(k))
                .and_then(Value::as_f64)
                .map(|n| n as f32)
        };
        let (Some(width), Some(height)) = (
            size(entry.get("size"), "width"),
            size(entry.get("size"), "height"),
        ) else {
            continue;
        };
        if width <= 0.0 || height <= 0.0 {
            continue;
        }
        let image = entry.get("image").and_then(|i| i.get("size"));
        let (out_width, out_height) = match (images, size(image, "width"), size(image, "height")) {
            // A page image has whole pixels; the JSON may carry rounding noise.
            (true, Some(px_w), Some(px_h)) if px_w >= 0.5 && px_h >= 0.5 => {
                (px_w.round(), px_h.round())
            }
            _ => (width, height),
        };
        frames.insert(
            number,
            PageFrame {
                height,
                scale_x: out_width / width,
                scale_y: out_height / height,
                out_width,
                out_height,
            },
        );
    }
    frames
}

/// One [`Anchor::Page`] per `prov` entry, in order (ADR 0011: all of them).
pub(crate) fn anchors(prov: &[&Value], frames: &BTreeMap<u32, PageFrame>) -> Vec<Anchor> {
    prov.iter()
        .filter_map(|p| {
            let page = p.get("page_no").and_then(Value::as_u64)? as u32;
            let bbox = frames.get(&page)?.bbox(p.get("bbox")?)?;
            Some(Anchor::Page { page, bbox })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bottom_left_boxes_turn_into_top_left() {
        let frame = PageFrame {
            height: 842.0,
            scale_x: 1.0,
            scale_y: 1.0,
            out_width: 595.0,
            out_height: 842.0,
        };
        let b = frame
            .bbox(&json!({"l": 36.0, "t": 800.0, "r": 150.0, "b": 780.0, "coord_origin": "BOTTOMLEFT"}))
            .unwrap();
        assert_eq!(
            (b.left, b.top, b.right, b.bottom),
            (36.0, 42.0, 150.0, 62.0)
        );
        let c = frame
            .bbox(&json!({"l": 10.0, "t": 20.0, "r": 30.0, "b": 40.0, "coord_origin": "TOPLEFT"}))
            .unwrap();
        assert_eq!((c.left, c.top, c.right, c.bottom), (10.0, 20.0, 30.0, 40.0));
    }

    #[test]
    fn image_pages_are_in_pixels_and_boxes_stay_inside() {
        let frame = PageFrame {
            height: 421.0,
            scale_x: 1240.0 / 297.6,
            scale_y: 1754.0 / 421.0,
            out_width: 1240.0,
            out_height: 1754.0,
        };
        let page = frame.page(1, true);
        assert_eq!((page.width, page.height), (1240.0, 1754.0));
        let b = frame
            .bbox(
                &json!({"l": -5.0, "t": 430.0, "r": 400.0, "b": 0.0, "coord_origin": "BOTTOMLEFT"}),
            )
            .unwrap();
        assert_eq!(b.left, 0.0);
        assert!(
            (b.right - 1240.0).abs() < 0.01 && b.top == 0.0 && (b.bottom - 1754.0).abs() < 0.01
        );
    }

    #[test]
    fn recognised_text_of_every_kind_sets_from_ocr() {
        let digital = TextOrigin {
            text_layer: true,
            ..TextOrigin::default()
        };
        assert!(!digital.is_recognised());
        assert!(
            TextOrigin {
                scanned_layer: true,
                ..digital
            }
            .is_recognised()
        );
        assert!(
            TextOrigin {
                recognised: true,
                ..digital
            }
            .is_recognised()
        );
        assert!(TextOrigin::default().is_recognised());
    }
}
