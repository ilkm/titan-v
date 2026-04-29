//! VM→guest-agent address table for control-plane capability hints (TOML-backed).

use std::collections::HashMap;
use std::net::SocketAddr;

/// In-memory binding map (schema compatible with `agent-bindings.toml` v1).
#[derive(Debug, Default, Clone)]
pub struct AgentBindingTable {
    inner: HashMap<String, SocketAddr>,
}

impl AgentBindingTable {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn insert(&mut self, vm: String, addr: SocketAddr) {
        self.inner.insert(vm, addr);
    }

    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&SocketAddr> {
        self.inner.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &SocketAddr)> {
        self.inner.iter()
    }
}
