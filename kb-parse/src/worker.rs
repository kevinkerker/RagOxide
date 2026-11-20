//! The worker process: converts one file with docling.rs and writes the result as JSON.
//!
//! Called by [`DoclingParser`](crate::DoclingParser) once per file as
//! `kb-parse-worker <file> <reply.json> [--trace]`, with the docling.rs settings in its
//! environment (README, "Worker-Prozess"). The reply goes to a file, not to standard
//! output, so that anything docling.rs prints cannot mix with it.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use kb_core::DocumentFormat;
use serde::{Deserialize, Serialize};

use crate::convert::{self, Converted, Failure};

/// What the worker writes into its reply file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Reply {
    /// The parsed content.
    Ok(Box<Converted>),
    /// Why the file could not be parsed.
    Err(Failure),
}

/// Entry point of the worker binary `kb-parse-worker`.
///
/// Exit code 0 when a reply was written (also for a file that could not be parsed),
/// 2 for wrong arguments or a reply that could not be written. A crash shows as any
/// other exit; the parser then reports [`kb_core::Error::Parse`].
pub fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let trace = args.iter().any(|a| a == "--trace");
    let paths: Vec<PathBuf> = args
        .iter()
        .filter(|a| *a != "--trace")
        .map(PathBuf::from)
        .collect();
    let [file, reply_path] = paths.as_slice() else {
        eprintln!("usage: kb-parse-worker <file> <reply.json> [--trace]");
        return ExitCode::from(2);
    };
    let reply = match DocumentFormat::from_path(file) {
        Some(format) => match convert::convert(file, format, trace) {
            Ok(converted) => Reply::Ok(Box::new(converted)),
            Err(failure) => Reply::Err(failure),
        },
        None => Reply::Err(Failure::Parse(format!(
            "unsupported format: {}",
            file.display()
        ))),
    };
    let written = std::fs::File::create(reply_path)
        .map_err(|e| e.to_string())
        .and_then(|f| {
            let mut out = std::io::BufWriter::new(f);
            serde_json::to_writer(&mut out, &reply).map_err(|e| e.to_string())?;
            std::io::Write::flush(&mut out).map_err(|e| e.to_string())
        });
    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "kb-parse-worker: cannot write {}: {e}",
                reply_path.display()
            );
            ExitCode::from(2)
        }
    }
}
