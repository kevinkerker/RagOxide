//! Requests to `llama-server` and the mapping of its answers onto the contract (F-F3).
//!
//! Formats from `docs/analyse/spike-llama-server-fortschritt.md` (sections 1, 3 and 4)
//! and `docs/analyse/tempomessung.md` ("Anfragen"), checked against the pinned release of the server.
//! Error texts never quote the request or the server's own messages, which might
//! repeat parts of the prompt (F35); they name status, error type and numbers only.

use kb_core::{AnswerPhase, Error, GenerationEvent, GenerationRequest, Message, PhaseProgress};
use serde::Serialize;
use serde_json::Value;

use crate::sse::{SseEvent, SseParser};

/// Body of `POST /v1/chat/completions`: the messages unchanged, `max_tokens` and
/// `temperature` only if set, progress while reading the prompt and no thinking
/// (ADR 0169).
#[derive(Debug, Serialize)]
pub(crate) struct ChatBody<'a> {
    messages: &'a [Message],
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    return_progress: bool,
    chat_template_kwargs: TemplateKwargs,
}

#[derive(Debug, Serialize)]
struct TemplateKwargs {
    enable_thinking: bool,
}

impl<'a> ChatBody<'a> {
    pub(crate) fn new(request: &'a GenerationRequest) -> Self {
        Self {
            messages: &request.messages,
            stream: true,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            return_progress: true,
            chat_template_kwargs: TemplateKwargs {
                enable_thinking: false,
            },
        }
    }
}

/// Body of `POST /tokenize`: without special tokens and without chat template.
#[derive(Debug, Serialize)]
pub(crate) struct TokenizeBody<'a> {
    pub(crate) content: &'a str,
    pub(crate) add_special: bool,
}

/// Number of tokens in the answer of `/tokenize` (`{"tokens":[...]}`).
pub(crate) fn token_count(body: &[u8]) -> Result<usize, Error> {
    let value: Value = serde_json::from_slice(body)
        .map_err(|_| Error::Generator("the answer of /tokenize is not JSON".into()))?;
    value
        .get("tokens")
        .and_then(Value::as_array)
        .map(Vec::len)
        .ok_or_else(|| Error::Generator("the answer of /tokenize has no token list".into()))
}

/// The error for an answer with a status other than 200, from the status and the JSON
/// error object of the server (F-F8: a prompt over the context is `InvalidInput`).
pub(crate) fn status_error(status: u16, body: &[u8]) -> Error {
    let error = serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|v| v.get("error").cloned());
    match error {
        Some(error) => server_error(Some(status), &error),
        None => Error::Generator(format!("the server answered HTTP {status}")),
    }
}

/// Maps an error object of the server.
fn server_error(status: Option<u16>, error: &Value) -> Error {
    let kind = error
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if kind == "exceed_context_size_error" {
        let number = |key: &str| error.get(key).and_then(Value::as_u64);
        return Error::InvalidInput(match (number("n_prompt_tokens"), number("n_ctx")) {
            (Some(prompt), Some(context)) => format!(
                "the prompt has {prompt} tokens and does not fit the context of {context} tokens"
            ),
            _ => "the prompt does not fit the context".into(),
        });
    }
    if kind == "authentication_error" {
        return Error::Generator("the server rejected the API key".into());
    }
    let code = error
        .get("code")
        .and_then(Value::as_u64)
        .map(|c| c.to_string())
        .or_else(|| status.map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".into());
    // `kind` is a fixed identifier of the server; keep it short and printable.
    let kind: String = kind
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(64)
        .collect();
    Error::Generator(format!(
        "the server reported an error (code {code}, type {kind})"
    ))
}

/// Turns the streamed answer of `/v1/chat/completions` into events of the contract.
#[derive(Debug, Default)]
pub(crate) struct ChatDecoder {
    sse: SseParser,
    events: Vec<SseEvent>,
    writing: bool,
    last_done: Option<u32>,
    finished: bool,
}

impl ChatDecoder {
    /// Whether `data: [DONE]` has arrived.
    pub(crate) fn finished(&self) -> bool {
        self.finished
    }

    /// Feeds the next bytes of the answer and appends the events they complete.
    pub(crate) fn push(
        &mut self,
        bytes: &[u8],
        out: &mut Vec<GenerationEvent>,
    ) -> Result<(), Error> {
        if self.finished {
            return Ok(());
        }
        self.sse
            .push(bytes, &mut self.events)
            .map_err(|_| Error::Generator("the server sent an event that is not UTF-8".into()))?;
        let events = std::mem::take(&mut self.events);
        for event in events {
            match event {
                SseEvent::Error(payload) => {
                    let value: Value = serde_json::from_str(&payload).unwrap_or(Value::Null);
                    let error = value.get("error").unwrap_or(&value);
                    return Err(server_error(None, error));
                }
                SseEvent::Data(payload) if payload == "[DONE]" => {
                    self.finished = true;
                    return Ok(());
                }
                SseEvent::Data(payload) => self.chunk(&payload, out)?,
            }
        }
        Ok(())
    }

    /// Checks the end of the answer: it must have ended with `[DONE]`.
    pub(crate) fn end(&self) -> Result<(), Error> {
        if self.finished {
            Ok(())
        } else if self.sse.has_partial() {
            Err(Error::Generator(
                "the answer of the server ended inside an event".into(),
            ))
        } else {
            Err(Error::Generator(
                "the answer of the server ended before it was complete".into(),
            ))
        }
    }

    fn chunk(&mut self, payload: &str, out: &mut Vec<GenerationEvent>) -> Result<(), Error> {
        let invalid =
            |what: &str| Error::Generator(format!("the server sent an invalid event: {what}"));
        let value: Value = serde_json::from_str(payload).map_err(|_| invalid("not JSON"))?;
        let Some(object) = value.as_object() else {
            return Err(invalid("not an object"));
        };
        if let Some(error) = object.get("error") {
            return Err(server_error(None, error));
        }
        let progress = object.get("prompt_progress");
        if let Some(progress) = progress {
            let number = |key: &str| progress.get(key).and_then(Value::as_u64);
            let (Some(processed), Some(total)) = (number("processed"), number("total")) else {
                return Err(invalid("progress without processed or total"));
            };
            self.progress(processed, total, out);
        }
        match object.get("choices") {
            Some(Value::Array(choices)) => {
                for choice in choices {
                    self.choice(choice, out).map_err(&invalid)?;
                }
            }
            Some(_) => return Err(invalid("choices is not a list")),
            None if progress.is_some() => {}
            None => return Err(invalid("no choices")),
        }
        Ok(())
    }

    fn progress(&mut self, processed: u64, total: u64, out: &mut Vec<GenerationEvent>) {
        if self.writing {
            return;
        }
        let total = u32::try_from(total).unwrap_or(u32::MAX);
        let done = u32::try_from(processed).unwrap_or(u32::MAX).min(total);
        if self.last_done.is_some_and(|last| done < last) {
            return;
        }
        self.last_done = Some(done);
        out.push(GenerationEvent::Phase {
            phase: AnswerPhase::ReadingSources,
            progress: Some(PhaseProgress { done, total }),
        });
    }

    fn choice(
        &mut self,
        choice: &Value,
        out: &mut Vec<GenerationEvent>,
    ) -> Result<(), &'static str> {
        let Some(delta) = choice.get("delta") else {
            return Ok(());
        };
        if !delta.is_object() {
            return Err("delta is not an object");
        }
        let text = |key: &str| match delta.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(text)) => Ok(Some(text.as_str())),
            Some(_) => Err("text field is not a string"),
        };
        let content = text("content")?;
        let reasoning = text("reasoning_content")?;
        if (content.is_some() || reasoning.is_some()) && !self.writing {
            self.writing = true;
            out.push(GenerationEvent::Phase {
                phase: AnswerPhase::Writing,
                progress: None,
            });
        }
        // Reasoning is hidden output: it opens `Writing` but never becomes `Text`.
        if let Some(content) = content.filter(|c| !c.is_empty()) {
            out.push(GenerationEvent::Text(content.to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use kb_core::{AnswerLevel, Role};

    use super::*;

    fn decode(text: &str) -> (Vec<GenerationEvent>, Result<(), Error>) {
        let mut decoder = ChatDecoder::default();
        let mut out = Vec::new();
        let result = decoder
            .push(text.as_bytes(), &mut out)
            .and_then(|()| decoder.end());
        (out, result)
    }

    fn reading(done: u32, total: u32) -> GenerationEvent {
        GenerationEvent::Phase {
            phase: AnswerPhase::ReadingSources,
            progress: Some(PhaseProgress { done, total }),
        }
    }

    const WRITING: GenerationEvent = GenerationEvent::Phase {
        phase: AnswerPhase::Writing,
        progress: None,
    };

    #[test]
    fn body_has_the_fields_of_f_f3() {
        let request = GenerationRequest {
            level: AnswerLevel::Fast,
            messages: vec![Message {
                role: Role::User,
                content: "Grüße \"x\" \\ \n".into(),
            }],
            max_tokens: None,
            temperature: Some(0.5),
        };
        let value = serde_json::to_value(ChatBody::new(&request)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "messages": [{"role": "user", "content": "Grüße \"x\" \\ \n"}],
                "stream": true,
                "temperature": 0.5,
                "return_progress": true,
                "chat_template_kwargs": {"enable_thinking": false}
            })
        );
    }

    #[test]
    fn stream_of_the_real_server_maps_onto_phases_and_text() {
        let stream = concat!(
            "data: {\"choices\":[{\"finish_reason\":null,\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":null}}],\"prompt_progress\":{\"total\":14,\"cache\":0,\"processed\":0,\"time_ms\":0}}\n\n",
            "data: {\"choices\":[{\"finish_reason\":null,\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":null}}],\"prompt_progress\":{\"total\":14,\"cache\":0,\"processed\":14,\"time_ms\":93}}\n\n",
            "data: {\"choices\":[{\"finish_reason\":null,\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":null}}]}\n\n",
            "data: {\"choices\":[{\"finish_reason\":null,\"index\":0,\"delta\":{\"content\":\"Hallo\"}}]}\n\n",
            "data: {\"choices\":[{\"finish_reason\":null,\"index\":0,\"delta\":{\"content\":\"\"}}]}\n\n",
            "data: {\"choices\":[{\"finish_reason\":null,\"index\":0,\"delta\":{\"content\":\"!\"}}]}\n\n",
            "data: {\"choices\":[{\"finish_reason\":\"stop\",\"index\":0,\"delta\":{}}],\"timings\":{\"predicted_n\":3}}\n\n",
            "data: [DONE]\n\n",
        );
        let (events, end) = decode(stream);
        end.unwrap();
        assert_eq!(
            events,
            vec![
                reading(0, 14),
                reading(14, 14),
                WRITING,
                GenerationEvent::Text("Hallo".into()),
                GenerationEvent::Text("!".into()),
            ]
        );
    }

    #[test]
    fn reasoning_opens_writing_but_is_never_text() {
        let stream = concat!(
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"hmm\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Ja\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let (events, end) = decode(stream);
        end.unwrap();
        assert_eq!(events, vec![WRITING, GenerationEvent::Text("Ja".into())]);
    }

    #[test]
    fn invalid_answers_are_generator_errors() {
        for stream in [
            "data: {not json\n\n",
            "data: {\"object\":\"chat.completion.chunk\"}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":5}}]}\n\n",
            "data: {\"prompt_progress\":{\"total\":3}}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"ab",
            "data: {\"choices\":[]}\n\n",
        ] {
            let (_, end) = decode(stream);
            assert!(matches!(end, Err(Error::Generator(_))), "{stream}: {end:?}");
        }
    }

    #[test]
    fn context_error_is_invalid_input_with_numbers() {
        let body = br#"{"error":{"code":400,"message":"request (1009 tokens) exceeds the available context size (512 tokens), try increasing it","type":"exceed_context_size_error","n_prompt_tokens":1009,"n_ctx":512}}"#;
        match status_error(400, body) {
            Error::InvalidInput(message) => {
                assert!(
                    message.contains("1009") && message.contains("512"),
                    "{message}"
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            status_error(401, br#"{"error":{"message":"Invalid API Key","type":"authentication_error","code":401}}"#),
            Error::Generator(_)
        ));
        assert!(matches!(status_error(500, b"oops"), Error::Generator(_)));
    }

    #[test]
    fn server_messages_are_not_quoted() {
        let body =
            br#"{"error":{"code":500,"message":"secret prompt text","type":"server_error"}}"#;
        let message = status_error(500, body).to_string();
        assert!(!message.contains("secret"), "{message}");
    }
}
