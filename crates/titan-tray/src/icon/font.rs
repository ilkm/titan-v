//! One-time system font pick for CJK + Latin tray glyphs (`控客CH`).

use std::sync::OnceLock;

use fontdb::{Database, Family, Query, Stretch, Style, Weight, ID};
use fontdue::{Font, FontSettings};

static TRAY_FONT: OnceLock<Option<Font>> = OnceLock::new();

const PROBE: &str = "控客CH";

pub(crate) fn tray_font() -> Option<&'static Font> {
    TRAY_FONT.get_or_init(load_tray_font).as_ref()
}

fn sans_bold_query() -> Query<'static> {
    Query {
        families: &[Family::SansSerif],
        weight: Weight::BOLD,
        stretch: Stretch::Normal,
        style: Style::Normal,
    }
}

fn ordered_face_ids(db: &Database, query: &Query<'_>) -> Vec<ID> {
    let mut ids = Vec::new();
    if let Some(id) = db.query(query) {
        ids.push(id);
    }
    for face in db.faces() {
        if !ids.contains(&face.id) {
            ids.push(face.id);
        }
    }
    ids
}

fn try_font_from_face(db: &Database, id: ID) -> Option<Font> {
    db.with_face_data(id, |data, idx| {
        Font::from_bytes(
            data,
            FontSettings {
                collection_index: idx,
                ..Default::default()
            },
        )
        .ok()
    })
    .flatten()
}

fn load_tray_font() -> Option<Font> {
    let mut db = Database::new();
    db.load_system_fonts();
    let query = sans_bold_query();
    let ids = ordered_face_ids(&db, &query);
    for id in ids {
        let Some(built) = try_font_from_face(&db, id) else {
            continue;
        };
        if font_covers_probe(&built) {
            return Some(built);
        }
    }
    None
}

fn font_covers_probe(font: &Font) -> bool {
    PROBE.chars().all(|c| font.has_glyph(c))
}
