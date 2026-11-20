//! Worker process of the parser: converts one file with docling.rs (see
//! `kb_parse::worker`).

fn main() -> std::process::ExitCode {
    kb_parse::worker::main()
}
