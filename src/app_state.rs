// Copyright 2019, Sjors van Gelderen

// use crate::attribute_table::AttributeTable;
// use crate::nametable::Nametable;
// use crate::palette::Palette;
use crate::pattern_table::PatternTable;

pub struct AppState {
    pub pattern_table: PatternTable,
}

impl AppState {
    pub fn zero() -> AppState {
        AppState {
            pattern_table: PatternTable::zero(),
        }
    }
}