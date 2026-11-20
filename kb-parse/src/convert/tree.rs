//! Reading order of the docling JSON (`export_to_json_value`): a walk of the item tree
//! from `body` that yields one [`Raw`] item per element candidate.
//!
//! Formats differ only in what they make of these items (anchors, layers, levels); the
//! walk itself is shared. Item references (`#/texts/3`) count per collection and are not
//! element indices (`docs/analyse/spike-parser-anker.md`, section 3).

use serde_json::Value;

use super::text::{collapse, nfc};

/// The docling JSON of one conversion.
pub(crate) struct DoclingJson<'a> {
    root: &'a Value,
}

impl<'a> DoclingJson<'a> {
    pub(crate) fn new(root: &'a Value) -> Self {
        Self { root }
    }

    /// The item a `$ref` such as `#/texts/3` points to.
    pub(crate) fn get(&self, reference: &str) -> Option<&'a Value> {
        let mut parts = reference.strip_prefix("#/")?.split('/');
        let collection = parts.next()?;
        match parts.next() {
            Some(index) => self.root.get(collection)?.get(index.parse::<usize>().ok()?),
            None => self.root.get(collection),
        }
    }

    /// The references of the top-level items in reading order.
    pub(crate) fn body(&self) -> Vec<&'a str> {
        self.get("#/body").map(children).unwrap_or_default()
    }

    /// The `pages` object (page number as string to page entry).
    pub(crate) fn pages(&self) -> Option<&'a serde_json::Map<String, Value>> {
        self.root.get("pages")?.as_object()
    }
}

/// The `$ref`s of an item's children, in order.
pub(crate) fn children(item: &Value) -> Vec<&str> {
    item.get("children")
        .and_then(Value::as_array)
        .map(|c| refs(c))
        .unwrap_or_default()
}

/// The `$ref`s of a list such as `captions` or `footnotes`.
pub(crate) fn refs(list: &[Value]) -> Vec<&str> {
    list.iter()
        .filter_map(|r| r.get("$ref").and_then(Value::as_str))
        .collect()
}

fn ref_list<'a>(item: &'a Value, key: &str) -> Vec<&'a str> {
    item.get(key)
        .and_then(Value::as_array)
        .map(|l| refs(l))
        .unwrap_or_default()
}

/// A string field of an item, empty if absent.
pub(crate) fn field<'a>(item: &'a Value, key: &str) -> &'a str {
    item.get(key).and_then(Value::as_str).unwrap_or("")
}

/// The collection of a reference: `texts`, `tables`, `pictures`, `groups`, ...
pub(crate) fn collection(reference: &str) -> &str {
    reference
        .strip_prefix("#/")
        .and_then(|r| r.split('/').next())
        .unwrap_or("")
}

/// What an element candidate is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawKind {
    /// A text item or a merged inline group; `label` says which.
    Text,
    /// A table item; its cells are in `item["data"]`.
    Table,
    /// A picture item; `text` holds the text found inside it.
    Picture,
}

/// One element candidate in reading order.
#[derive(Debug, Clone)]
pub(crate) struct Raw<'a> {
    /// The docling item (for `prov`, `level` and table data).
    pub item: &'a Value,
    /// Its reference, e.g. `#/texts/12`.
    pub self_ref: &'a str,
    /// What the candidate is.
    pub kind: RawKind,
    /// The docling label (`text`, `section_header`, `title`, `caption`, ...).
    pub label: &'a str,
    /// The content layer (`body`, `furniture`, `notes`, `invisible`).
    pub layer: &'a str,
    /// The text, NFC.
    pub text: String,
    /// The slide of the enclosing `slide-N` chapter group (PPTX), 1-based.
    pub slide: Option<u32>,
    /// Location records (`prov`) of the item, or of its descendants if it has none.
    pub prov: Vec<&'a Value>,
}

/// Walks the tree from `body` and returns the element candidates in reading order.
pub(crate) fn walk<'a>(json: &DoclingJson<'a>) -> Vec<Raw<'a>> {
    let mut out = Vec::new();
    for reference in json.body() {
        visit(json, reference, None, &mut out);
    }
    out
}

/// The text of a text item: `text`; for an item labelled `formula`, which docling.rs
/// leaves without `text`, its `orig`. Other items keep an empty `text` (docling.rs
/// empties pieces that hold only invisible characters such as U+FEFF).
fn item_text(item: &Value) -> &str {
    match field(item, "text") {
        text if text.trim().is_empty() && field(item, "label") == "formula" => field(item, "orig"),
        text => text,
    }
}

fn layer(item: &Value) -> &str {
    match field(item, "content_layer") {
        "" => "body",
        layer => layer,
    }
}

fn visit<'a>(
    json: &DoclingJson<'a>,
    reference: &'a str,
    slide: Option<u32>,
    out: &mut Vec<Raw<'a>>,
) {
    let Some(item) = json.get(reference) else {
        return;
    };
    let label = field(item, "label");
    match collection(reference) {
        "groups" => {
            if label == "inline" {
                out.push(Raw {
                    item,
                    self_ref: reference,
                    kind: RawKind::Text,
                    label: "text",
                    layer: layer(item),
                    text: collapse(&inline_text(json, item)),
                    slide,
                    prov: prov_of(json, item),
                });
                return;
            }
            let slide = match label {
                "chapter" => field(item, "name")
                    .strip_prefix("slide-")
                    .and_then(|n| n.parse::<u32>().ok())
                    .map(|n| n + 1)
                    .or(slide),
                _ => slide,
            };
            for child in children(item) {
                visit(json, child, slide, out);
            }
        }
        "texts" => {
            let own = item_text(item);
            let kids = children(item);
            // A list item or paragraph whose content docling put into inline groups
            // (HTML with links): the groups are its text.
            let (text, rest): (String, Vec<&str>) = if own.trim().is_empty() {
                let (inline, rest): (Vec<&str>, Vec<&str>) = kids.iter().partition(|r| {
                    json.get(r)
                        .is_some_and(|c| collection(r) == "groups" && field(c, "label") == "inline")
                });
                let text = inline
                    .iter()
                    .filter_map(|r| json.get(r))
                    .map(|g| inline_text(json, g))
                    .collect::<Vec<_>>()
                    .join(" ");
                (collapse(&text), rest)
            } else {
                (nfc(own), kids)
            };
            out.push(Raw {
                item,
                self_ref: reference,
                kind: RawKind::Text,
                label,
                layer: layer(item),
                text,
                slide,
                prov: prov_of(json, item),
            });
            for child in rest {
                visit(json, child, slide, out);
            }
        }
        "tables" => {
            for caption in ref_list(item, "captions") {
                visit(json, caption, slide, out);
            }
            out.push(Raw {
                item,
                self_ref: reference,
                kind: RawKind::Table,
                label,
                layer: layer(item),
                text: String::new(),
                slide,
                prov: prov_of(json, item),
            });
            // The other children are the contents of rich cells, already in the cells.
            for footnote in ref_list(item, "footnotes") {
                visit(json, footnote, slide, out);
            }
        }
        "pictures" => {
            let captions = ref_list(item, "captions");
            let footnotes = ref_list(item, "footnotes");
            for caption in &captions {
                visit(json, caption, slide, out);
            }
            let mut texts = Vec::new();
            for child in children(item) {
                if !captions.contains(&child) && !footnotes.contains(&child) {
                    collect_texts(json, child, &mut texts);
                }
            }
            let texts: Vec<String> = texts
                .iter()
                .map(|t| collapse(t))
                .filter(|t| !t.is_empty())
                .collect();
            out.push(Raw {
                item,
                self_ref: reference,
                kind: RawKind::Picture,
                label,
                layer: layer(item),
                text: texts.join("\n"),
                slide,
                prov: prov_of(json, item),
            });
            for footnote in footnotes {
                visit(json, footnote, slide, out);
            }
        }
        _ => {
            for child in children(item) {
                visit(json, child, slide, out);
            }
        }
    }
}

/// The text of an inline group: its pieces as written, including their own spacing. For
/// HTML the package puts a space at every block boundary before docling.rs reads the
/// page (see `html.rs`), so pieces of separate blocks carry one; pieces of one word stay
/// together.
fn inline_text(json: &DoclingJson<'_>, group: &Value) -> String {
    let mut pieces = Vec::new();
    for child in children(group) {
        collect_texts(json, child, &mut pieces);
    }
    pieces.concat()
}

/// The texts of an item and its descendants, depth first.
fn collect_texts(json: &DoclingJson<'_>, reference: &str, out: &mut Vec<String>) {
    let Some(item) = json.get(reference) else {
        return;
    };
    if collection(reference) == "texts" {
        out.push(item_text(item).to_string());
    }
    for child in children(item) {
        collect_texts(json, child, out);
    }
}

/// The `prov` entries of an item; for an item without any (a merged inline group) those
/// of its descendants in reading order.
fn prov_of<'a>(json: &DoclingJson<'a>, item: &'a Value) -> Vec<&'a Value> {
    let own: Vec<&Value> = item
        .get("prov")
        .and_then(Value::as_array)
        .map(|p| p.iter().collect())
        .unwrap_or_default();
    if !own.is_empty() {
        return own;
    }
    let mut prov = Vec::new();
    for child in children(item) {
        if let Some(c) = json.get(child) {
            prov.extend(prov_of(json, c));
        }
    }
    prov
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn inline_groups_become_one_paragraph() {
        let doc = json!({
            "body": {"children": [{"$ref": "#/groups/0"}]},
            "groups": [{"label": "inline", "content_layer": "body", "children": [
                {"$ref": "#/texts/0"}, {"$ref": "#/texts/1"}, {"$ref": "#/texts/2"}]}],
            "texts": [
                {"label": "text", "text": "Der Tag wurde vom "},
                {"label": "text", "text": "Bundespräsidenten"},
                {"label": "text", "text": " festgelegt."}
            ]
        });
        let json = DoclingJson::new(&doc);
        let raws = walk(&json);
        assert_eq!(raws.len(), 1);
        assert_eq!(
            raws[0].text,
            "Der Tag wurde vom Bundespräsidenten festgelegt."
        );
    }

    #[test]
    fn formula_text_comes_from_orig() {
        let doc = json!({
            "body": {"children": [{"$ref": "#/texts/0"}]},
            "texts": [{"label": "formula", "text": "", "orig": "(Lachen bei der AfD)"}]
        });
        let json = DoclingJson::new(&doc);
        let raws = walk(&json);
        assert_eq!(raws[0].text, "(Lachen bei der AfD)");
        // Other labels keep an empty text: no invisible U+FEFF comes back from `orig`.
        let doc = json!({
            "body": {"children": [{"$ref": "#/texts/0"}]},
            "texts": [{"label": "list_item", "text": "", "orig": "\u{feff}"}]
        });
        let json = DoclingJson::new(&doc);
        assert_eq!(walk(&json)[0].text, "");
    }

    #[test]
    fn picture_text_comes_from_its_children_and_captions_stand_before() {
        let doc = json!({
            "body": {"children": [{"$ref": "#/pictures/0"}]},
            "pictures": [{"label": "picture", "captions": [{"$ref": "#/texts/0"}],
                "children": [{"$ref": "#/texts/0"}, {"$ref": "#/texts/1"}]}],
            "texts": [
                {"label": "caption", "text": "Figure 1"},
                {"label": "text", "text": "axis label"}
            ]
        });
        let json = DoclingJson::new(&doc);
        let raws = walk(&json);
        let shown: Vec<(&str, &str)> = raws.iter().map(|r| (r.label, r.text.as_str())).collect();
        assert_eq!(shown, [("caption", "Figure 1"), ("picture", "axis label")]);
    }
}
