//! Shared UI language for center and host (persistence + control-plane).

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Display language for shared desktop UIs (`en` / `zh` in JSON and SQLite).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Serialize,
    Deserialize,
    Default,
)]
#[serde(rename_all = "lowercase")]
pub enum UiLang {
    #[default]
    En,
    Zh,
}
