//! Script engine smoke test via `ScriptEngine` (same path as `titan_host::batch::script_eval`).

use std::io::Write;

use tempfile::NamedTempFile;

#[test]
fn eval_chunk_writes_temp_file() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "local x = 1 + 2").unwrap();
    let p = f.path();
    let src = std::fs::read_to_string(p).unwrap();
    let eng = titan_scripts::ScriptEngine::new().unwrap();
    eng.exec_chunk(&src).unwrap();
}
