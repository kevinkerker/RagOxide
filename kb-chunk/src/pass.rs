//! Counting, packing and splitting of running text during one `chunk` call.

use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::task::{Context, Poll};

use kb_core::{Error, Result, TokenCounter};

use crate::sentence;

/// Result of packing items into runs.
pub(crate) enum Packed {
    /// Items `start..end` whose rendered `text` fits.
    Fit {
        start: usize,
        end: usize,
        text: String,
    },
    /// Item `index` does not fit even on its own; `tokens` is its rendered count.
    TooLarge { index: usize, tokens: usize },
}

/// Counter and limit of one `chunk` call.
pub(crate) struct Pass<'a> {
    counter: &'a dyn TokenCounter,
    limit: usize,
}

impl<'a> Pass<'a> {
    pub(crate) fn new(counter: &'a dyn TokenCounter, limit: usize) -> Self {
        Self { counter, limit }
    }

    pub(crate) fn limit(&self) -> usize {
        self.limit
    }

    /// Counts `text` and then yields once, so that a long document does not hold the
    /// executor even if the counter never waits. Errors of the counter pass unchanged.
    pub(crate) async fn count(&mut self, text: &str) -> Result<usize> {
        if text.is_empty() {
            return Ok(0);
        }
        let tokens = self.counter.count_tokens(text).await?;
        YieldNow(false).await;
        Ok(tokens)
    }

    /// Splits the items `0..len` into consecutive runs, each as long as fits.
    ///
    /// `render(start, end)` is the text of items `start..end`, `estimate(start, end)` a
    /// guess of its tokens that only steers which lengths are counted: every emitted
    /// text is counted. From the guess the search gallops forward or bisects back, so a
    /// run usually costs two counts. With `single_fits` every item is known to fit on its
    /// own; otherwise an item that does not is reported as [`Packed::TooLarge`].
    pub(crate) async fn pack(
        &mut self,
        len: usize,
        single_fits: bool,
        estimate: impl Fn(usize, usize) -> usize,
        render: impl Fn(usize, usize) -> String,
    ) -> Result<Vec<Packed>> {
        let mut packed = Vec::new();
        let mut start = 0;
        while start < len {
            // Longest end known to fit (with its text, if rendered) and shortest end
            // known not to fit (with its count).
            let mut fit = Probe {
                end: if single_fits { start + 1 } else { start },
                text: None,
            };
            let mut over: Option<(usize, usize)> = None;
            let mut guess = start + 1;
            while guess < len && estimate(start, guess + 1) <= self.limit {
                guess += 1;
            }
            if guess > fit.end {
                self.probe(start, guess, &render, &mut fit, &mut over)
                    .await?;
            }
            let mut step = 1;
            while over.is_none() && fit.end < len {
                let end = fit.end.saturating_add(step).min(len);
                self.probe(start, end, &render, &mut fit, &mut over).await?;
                step = step.saturating_mul(2);
            }
            while let Some((end, _)) = over
                && end > fit.end + 1
            {
                let middle = fit.end + (end - fit.end) / 2;
                self.probe(start, middle, &render, &mut fit, &mut over)
                    .await?;
            }
            if fit.end == start {
                let tokens = over.map_or(0, |(_, tokens)| tokens);
                packed.push(Packed::TooLarge {
                    index: start,
                    tokens,
                });
                start += 1;
            } else {
                let end = fit.end;
                let text = fit.text.unwrap_or_else(|| render(start, end));
                packed.push(Packed::Fit { start, end, text });
                start = end;
            }
        }
        Ok(packed)
    }

    async fn probe(
        &mut self,
        start: usize,
        end: usize,
        render: &impl Fn(usize, usize) -> String,
        fit: &mut Probe,
        over: &mut Option<(usize, usize)>,
    ) -> Result<()> {
        let text = render(start, end);
        let tokens = self.count(&text).await?;
        if tokens <= self.limit {
            if end > fit.end {
                *fit = Probe {
                    end,
                    text: Some(text),
                };
            }
        } else if over.is_none_or(|(known, _)| end < known) {
            *over = Some((end, tokens));
        }
        Ok(())
    }

    /// Splits running text of `tokens` tokens that exceeds the limit: at sentence
    /// boundaries, failing that at word boundaries, failing that at character
    /// boundaries. Pieces are trimmed slices of `text`.
    pub(crate) async fn split_text(&mut self, text: &str, tokens: usize) -> Result<Vec<String>> {
        let sentences = sentence::sentences(text);
        let mut pieces = Vec::new();
        for packed in self.pack_ranges(text, &sentences, tokens).await? {
            match packed {
                Packed::Fit { text: piece, .. } => pieces.push(piece),
                Packed::TooLarge { index, tokens } => {
                    let sentence = &text[sentences[index].clone()];
                    pieces.extend(self.split_words(sentence, tokens).await?);
                }
            }
        }
        Ok(pieces)
    }

    async fn split_words(&mut self, text: &str, tokens: usize) -> Result<Vec<String>> {
        let words = words(text);
        let mut pieces = Vec::new();
        for packed in self.pack_ranges(text, &words, tokens).await? {
            match packed {
                Packed::Fit { text: piece, .. } => pieces.push(piece),
                Packed::TooLarge { index, tokens } => {
                    let word = &text[words[index].clone()];
                    pieces.extend(self.split_chars(word, tokens).await?);
                }
            }
        }
        Ok(pieces)
    }

    /// Token boundaries as the package reads them: the longest prefix that fits, found
    /// by counting prefixes, never inside a character.
    async fn split_chars(&mut self, word: &str, tokens: usize) -> Result<Vec<String>> {
        let chars: Vec<Range<usize>> = word
            .char_indices()
            .map(|(i, c)| i..i + c.len_utf8())
            .collect();
        let mut pieces = Vec::new();
        for packed in self.pack_ranges(word, &chars, tokens).await? {
            match packed {
                Packed::Fit { text: piece, .. } => pieces.push(piece),
                Packed::TooLarge { tokens, .. } => {
                    return Err(Error::Chunk(format!(
                        "a single character has {tokens} tokens, more than the limit of {}",
                        self.limit
                    )));
                }
            }
        }
        Ok(pieces)
    }

    /// Packs consecutive byte ranges of `text` (sentences, words, characters) into
    /// slices that fit, guessing their tokens in proportion to their length.
    async fn pack_ranges(
        &mut self,
        text: &str,
        ranges: &[Range<usize>],
        tokens: usize,
    ) -> Result<Vec<Packed>> {
        let estimate = |start: usize, end: usize| {
            scale(
                ranges[end - 1].end - ranges[start].start,
                tokens,
                text.len(),
            )
        };
        let render =
            |start: usize, end: usize| text[ranges[start].start..ranges[end - 1].end].to_owned();
        self.pack(ranges.len(), false, estimate, render).await
    }
}

/// The longest end known to fit during a search, with its text once rendered.
struct Probe {
    end: usize,
    text: Option<String>,
}

/// `part` bytes of a text with `total` bytes and `tokens` tokens, in tokens, rounded up.
pub(crate) fn scale(part: usize, tokens: usize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let scaled = (part as u128 * tokens as u128).div_ceil(total as u128);
    usize::try_from(scaled).unwrap_or(usize::MAX)
}

/// Byte ranges of the whitespace-separated words of `text`.
fn words(text: &str) -> Vec<Range<usize>> {
    let mut words = Vec::new();
    let mut start = None;
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            if let Some(s) = start.take() {
                words.push(s..i);
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        words.push(s..text.len());
    }
    words
}

/// Returns `Pending` once and wakes itself, so the executor can run other tasks.
struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
