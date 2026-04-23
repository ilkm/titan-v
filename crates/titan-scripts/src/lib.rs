//! Lua automation host built on **mlua**.
//!
//! Sandboxing, capability whitelists, and timeouts will be layered here as the project matures.

#![forbid(unsafe_code)]

use mlua::{Lua, StdLib};
use thiserror::Error;

/// Script engine errors (never embed user script source in `Display` for production logs).
#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("lua runtime: {0}")]
    Lua(#[from] mlua::Error),
}

pub type Result<T> = std::result::Result<T, ScriptError>;

/// Owns a Lua VM with a conservative standard library surface.
pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    /// Creates a new Lua 5.4 VM (bundled via mlua `vendored`).
    pub fn new() -> Result<Self> {
        let lua = Lua::new_with(
            StdLib::STRING | StdLib::TABLE | StdLib::MATH,
            mlua::LuaOptions::new(),
        )?;
        Ok(Self { lua })
    }

    /// Executes a chunk and discards its return values.
    pub fn exec_chunk(&self, source: &str) -> Result<()> {
        self.lua.load(source).set_name("chunk").exec()?;
        Ok(())
    }

    /// Evaluates a chunk that returns a single value (e.g. `return 1 + 1`).
    pub fn eval_i64(&self, source: &str) -> Result<i64> {
        let v: i64 = self.lua.load(source).set_name("expr").eval()?;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_simple_expression() {
        let eng = ScriptEngine::new().unwrap();
        assert_eq!(eng.eval_i64("return 1 + 1").unwrap(), 2);
    }
}
