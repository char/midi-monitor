use crate::note::{SpellingContext, scale_intervals};

use super::{
    intervals::{bit, bits},
    symbol::ChordBase,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Template {
    pub base: ChordBase,
    pub required: u16,
    pub optional: u16,
    pub rank: i32,
}

pub const TEMPLATES: &[Template] = &[
    template(ChordBase::Major, &[4], &[7], 0),
    template(ChordBase::Minor, &[3], &[7], 0),
    template(ChordBase::Five, &[7], &[], 16),
    template(ChordBase::Diminished, &[3, 6], &[], 10),
    template(ChordBase::Augmented, &[4, 8], &[7], 14),
    template(ChordBase::Sus2, &[2, 7], &[], 8),
    template(ChordBase::Sus4, &[5], &[2, 7], 6),
    template(ChordBase::SixSus4, &[5, 7, 9], &[2], 18),
    template(ChordBase::Six, &[4, 9], &[7], 4),
    template(ChordBase::MinorSix, &[3, 9], &[7], 4),
    template(ChordBase::SixNine, &[4, 9, 2], &[7], 2),
    template(ChordBase::MinorSixNine, &[3, 9, 2], &[7], 2),
    template(ChordBase::Seven, &[4, 10], &[7], 0),
    template(ChordBase::MajorSeven, &[4, 11], &[7], 0),
    template(ChordBase::MinorSeven, &[3, 10], &[7], 0),
    template(ChordBase::MinorMajorSeven, &[3, 11], &[7], 10),
    template(ChordBase::HalfDiminishedSeven, &[3, 6, 10], &[], 0),
    template(ChordBase::DiminishedSeven, &[3, 6, 9], &[], 0),
    template(ChordBase::Nine, &[4, 10, 2], &[7], 0),
    template(ChordBase::MajorNine, &[4, 11, 2], &[7], 0),
    template(ChordBase::MinorNine, &[3, 10, 2], &[7], 0),
    template(ChordBase::Eleven, &[4, 10, 2, 5], &[7], 6),
    template(ChordBase::MinorEleven, &[3, 10, 2, 5], &[7], 4),
    template(ChordBase::Thirteen, &[4, 10, 9, 2], &[7], 4),
    template(ChordBase::ThirteenFlatNine, &[4, 10, 9, 1], &[7], 2),
    template(ChordBase::ThirteenSharpNine, &[4, 10, 9, 3], &[7], 4),
    template(ChordBase::MajorThirteen, &[4, 11, 9, 2], &[7], 6),
    template(ChordBase::MinorThirteen, &[3, 10, 9, 2], &[7], 8),
    template(ChordBase::SevenSus4, &[5, 10], &[7], 0),
    template(ChordBase::NineSus4, &[5, 10, 2], &[7], 0),
    template(ChordBase::ThirteenSus4, &[5, 10, 9, 2], &[7], 4),
];

const fn template(base: ChordBase, required: &[usize], optional: &[usize], rank: i32) -> Template {
    Template {
        base,
        required: bits(required) | bit(0),
        optional: bits(optional),
        rank,
    }
}

pub fn scale_triad(root: usize, spelling: &SpellingContext) -> Option<ChordBase> {
    let intervals = scale_intervals(spelling.scale);
    if intervals.len() != 7 {
        return None;
    }

    let root = root % 12;
    let degree = intervals
        .iter()
        .position(|interval| (spelling.root_pitch_class + interval) % 12 == root)?;
    let scale_pitch_class = |offset| (spelling.root_pitch_class + intervals[(degree + offset) % 7]) % 12;
    let third = (scale_pitch_class(2) + 12 - root) % 12;
    let fifth = (scale_pitch_class(4) + 12 - root) % 12;

    match (third, fifth) {
        (4, 7) => Some(ChordBase::Major),
        (3, 7) => Some(ChordBase::Minor),
        (3, 6) => Some(ChordBase::Diminished),
        (4, 8) => Some(ChordBase::Augmented),
        _ => None,
    }
}
