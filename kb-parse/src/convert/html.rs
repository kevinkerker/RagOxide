//! HTML: the main content of a page, like a browser's reader mode (ADR 0096).
//!
//! docling.rs reads the whole page and treats everything before the first heading as
//! furniture. The package hands it the main content only and reads that completely:
//!
//! - The main part is the first `<main>` or element with `role="main"`, else the first
//!   `<article>`, else `<body>`.
//! - Inside it, navigation and side content are removed: `<nav>`, `<aside>`, and
//!   elements with `role` `navigation`, `complementary`, `search`, `banner` or
//!   `contentinfo`. If the main part is `<body>` (the page names none), the page's
//!   `<header>` and `<footer>` are removed as well; inside an explicit `<main>` or
//!   `<article>` they belong to the content.
//! - The page title (`<title>` in the head) becomes the document title only (ADR 0135);
//!   docling.rs does not get it, so it is no element. A `<title>` inside an SVG of the
//!   page body is neither title nor element; it is removed with its text.
//! - A space is put at every boundary of a block element and after `<br>` (see
//!   [`space_blocks`]): docling.rs joins the pieces of an inline group as written, so text
//!   of separate blocks keeps a space between, and text of one word across inline elements
//!   (`H<sub>2</sub>O`, `Bundes<b>tags</b>wahl`) stays together.

use encoding_rs::{Encoding, WINDOWS_1252};
use scraper::{ElementRef, Html, Selector};

/// Decodes the bytes of an HTML file the way docling.rs reads HTML (its
/// `decode_html_bytes`, after BeautifulSoup): a byte order mark wins; else the encoding
/// the file declares (`encoding=` of an XML declaration in the first 1024 bytes, else the
/// first `charset=` of a `<meta>` tag in the first `max(2048, 5 % of the length)` bytes);
/// else UTF-8 if the bytes are valid UTF-8; else windows-1252. A candidate is taken only
/// if it decodes without error. Labels follow the WHATWG table, so `iso-8859-1` reads as
/// windows-1252, as in browsers.
pub(crate) fn decode(bytes: &[u8]) -> String {
    if let Some((encoding, bom)) = Encoding::for_bom(bytes)
        && let Some(text) =
            encoding.decode_without_bom_handling_and_without_replacement(&bytes[bom..])
    {
        return text.into_owned();
    }
    if let Some(encoding) = declared_encoding(bytes).and_then(|l| Encoding::for_label(l.as_bytes()))
        && let Some(text) = encoding.decode_without_bom_handling_and_without_replacement(bytes)
    {
        return text.into_owned();
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => WINDOWS_1252
            .decode_without_bom_handling(bytes)
            .0
            .into_owned(),
    }
}

/// The encoding label an HTML file declares, lower case, see [`decode`].
fn declared_encoding(bytes: &[u8]) -> Option<String> {
    let lower = |range: &[u8]| String::from_utf8_lossy(range).to_ascii_lowercase();
    let head = lower(&bytes[..bytes.len().min(1024)]);
    if head.trim_start().starts_with("<?")
        && let Some(end) = head.find("?>")
        && let Some(label) = attribute_value(&head[..end], "encoding=")
    {
        return Some(label);
    }
    let html = lower(&bytes[..bytes.len().min(2048.max(bytes.len() / 20))]);
    let mut rest = html.as_str();
    while let Some(start) = rest.find("<meta") {
        let tag = &rest[start..];
        let tag = &tag[..tag.find('>').unwrap_or(tag.len())];
        if let Some(label) = attribute_value(tag, "charset=") {
            return Some(label);
        }
        rest = &rest[start + 5..];
    }
    None
}

/// The value after `key` (`charset=` or `encoding=`), without quotes, up to the next
/// space, `;`, `/`, quote or end.
fn attribute_value(text: &str, key: &str) -> Option<String> {
    let value = text[text.find(key)? + key.len()..].trim_start();
    let value = value.trim_start_matches(['"', '\'']);
    let end = value
        .find(|c: char| c.is_whitespace() || matches!(c, ';' | '/' | '"' | '\'' | '>'))
        .unwrap_or(value.len());
    let label = value[..end].trim();
    (!label.is_empty()).then(|| label.to_string())
}

/// The main content of `html` as a new HTML document, and the page title.
pub(crate) fn main_content(html: &str) -> (String, Option<String>) {
    let mut doc = Html::parse_document(html);
    let select = |css: &str| Selector::parse(css).expect("static selector");
    let title = doc
        .select(&select("head > title"))
        .next()
        .map(|t| {
            t.text()
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|t| !t.is_empty());
    let explicit = doc
        .select(&select("main, [role=main]"))
        .next()
        .or_else(|| doc.select(&select("article")).next())
        .map(|e| e.id());
    let root = explicit.or_else(|| doc.select(&select("body")).next().map(|e| e.id()));
    let Some(root) = root else {
        return (html.to_string(), title);
    };
    let mut remove = String::from(
        "nav, aside, [role=navigation], [role=complementary], [role=search], [role=banner], \
         [role=contentinfo], svg title",
    );
    if explicit.is_none() {
        remove.push_str(", header, footer");
    }
    let doomed: Vec<_> = doc
        .tree
        .get(root)
        .and_then(ElementRef::wrap)
        .map(|r| r.select(&select(&remove)).map(|e| e.id()).collect())
        .unwrap_or_default();
    for id in doomed {
        if let Some(mut node) = doc.tree.get_mut(id) {
            node.detach();
        }
    }
    let inner = doc
        .tree
        .get(root)
        .and_then(ElementRef::wrap)
        .map(|r| space_blocks(&r.inner_html()))
        .unwrap_or_default();
    (
        format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>{inner}</body></html>"
        ),
        title,
    )
}

/// Elements that start a new block in a browser, and `<br>`.
const BLOCKS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "br",
    "caption",
    "dd",
    "details",
    "dialog",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

/// Puts a space before every opening and after every closing tag of a block element in
/// serialised HTML (where every `<` starts a tag or comment, text has `&lt;`).
fn space_blocks(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + html.len() / 16);
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let tag = &rest[start..];
        let end = tag.find('>').map_or(tag.len(), |e| e + 1);
        let closing = tag[1..].starts_with('/');
        let name: String = tag[1 + usize::from(closing)..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        let block = BLOCKS.contains(&name.as_str());
        if block && !closing {
            out.push(' ');
        }
        out.push_str(&tag[..end]);
        if block && (closing || name == "br") {
            out.push(' ');
        }
        rest = &tag[end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_and_bom_encodings_are_read() {
        let latin1 = b"<html><head><meta charset=\"iso-8859-1\"><title>Gr\xf6\xdfe</title></head><body><p>M\xe4ngel</p></body></html>";
        let text = decode(latin1);
        assert!(text.contains("Gr\u{f6}\u{df}e") && text.contains("M\u{e4}ngel"));
        let cp1252 = b"<meta http-equiv=\"Content-Type\" content=\"text/html; charset=windows-1252\"><p>\x84Zitat\x93 \x80</p>";
        assert!(decode(cp1252).contains("\u{201E}Zitat\u{201C} \u{20AC}"));
        let bom = b"\xef\xbb\xbf<meta charset=\"iso-8859-1\"><p>\xc3\xa4</p>";
        assert!(decode(bom).contains("<p>\u{e4}</p>"));
        assert!(decode("<p>schon UTF-8: \u{fc}</p>".as_bytes()).contains('\u{fc}'));
    }

    #[test]
    fn blocks_are_spaced_and_words_stay_whole() {
        let html = "<div>\u{2190} 2021</div><div>Wahl</div>H<sub>2</sub>O Bundes<b>tags</b>wahl \
                    10<sup>3</sup> Text<sup>[1]</sup><br>neu<p>Absatz</p><pre>x</pre>";
        let spaced = space_blocks(html);
        assert_eq!(
            spaced,
            " <div>\u{2190} 2021</div>  <div>Wahl</div> H<sub>2</sub>O Bundes<b>tags</b>wahl \
                    10<sup>3</sup> Text<sup>[1]</sup> <br> neu <p>Absatz</p>  <pre>x</pre> "
        );
    }

    #[test]
    fn a_title_inside_an_svg_is_no_title() {
        let page =
            "<html><head></head><body><svg><title>Grafik</title></svg><p>Text</p></body></html>";
        let (html, title) = main_content(page);
        assert_eq!(title, None);
        assert!(!html.contains("Grafik") && html.contains("Text"));
    }

    #[test]
    fn only_the_main_part_remains() {
        let page = "<html><head><title>Seite</title></head><body>\
            <header><nav>Menü</nav></header><nav>Navi</nav>\
            <main><p>Inhalt</p><aside>Rand</aside><div role=\"navigation\">Siehe auch</div></main>\
            <aside>Werbung</aside><footer>Impressum</footer></body></html>";
        let (html, title) = main_content(page);
        assert_eq!(title.as_deref(), Some("Seite"));
        assert!(html.contains("Inhalt"));
        assert!(
            !html.contains("<title>"),
            "the title is no element (ADR 0135)"
        );
        for gone in ["Menü", "Navi", "Rand", "Siehe auch", "Werbung", "Impressum"] {
            assert!(!html.contains(gone), "{gone} in {html}");
        }
    }

    #[test]
    fn without_main_the_page_header_and_footer_go() {
        let page = "<html><body><header>Kopf</header><p>Text vor der Überschrift</p>\
            <h2>Teil</h2><p>Mehr</p><footer>Fuß</footer></body></html>";
        let (html, title) = main_content(page);
        assert_eq!(title, None);
        assert!(html.contains("Text vor der Überschrift") && html.contains("Mehr"));
        assert!(!html.contains("Kopf") && !html.contains("Fuß"));
    }
}
