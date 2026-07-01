use crate::note::{SpellingContext, scale_intervals};

use super::{
    detect::{FormulaMatch, Notes},
    intervals::{Intervals, bit, has_seventh},
    symbol::{ChordBase, ChordSuffix, Modifier},
};

const MISSING_FORCED_ROOT: i32 = 8;
const MISSING_INFERRED_ROOT: i32 = 24;
const MISSING_ESSENTIAL: i32 = 26;
const MISSING_ESSENTIAL_IN_SPARSE_CHORD: i32 = 32;
const MISSING_SEVENTH: i32 = 44;
const PRESENT_ESSENTIAL_BONUS: i32 = -9;
const PRESENT_OPTIONAL_BONUS: i32 = -3;
const MISSING_OPTIONAL: i32 = 5;
const ABSENT_FORCED_ROOT: i32 = 8;

pub fn score_candidate(matched: &FormulaMatch<'_>, forced: bool, spelling: Option<&SpellingContext>) -> i32 {
    let missing_essentials = (matched.missing_required & !bit(0)).count_ones() as i32;
    let missing_root = when(
        matched.missing_required & bit(0) != 0,
        if forced {
            MISSING_FORCED_ROOT
        } else {
            MISSING_INFERRED_ROOT
        },
    );
    let missing_seventh = when(
        has_seventh(matched.template.required) && matched.missing_required & (bit(10) | bit(11)) != 0,
        MISSING_SEVENTH,
    );
    let optional_support = matched.present_optional.count_ones() as i32 * PRESENT_OPTIONAL_BONUS
        + matched.missing_optional.count_ones() as i32 * MISSING_OPTIONAL;

    matched.template.rank
        + missing_root
        + missing_essentials * MISSING_ESSENTIAL
        + when(
            matched.notes.pitch_classes.len() <= 2,
            missing_essentials * MISSING_ESSENTIAL_IN_SPARSE_CHORD,
        )
        + missing_seventh
        + (matched.present_required & !bit(0)).count_ones() as i32 * PRESENT_ESSENTIAL_BONUS
        + optional_support
        + extra_cost(matched.extras, matched.intervals, &matched.suffix)
        + voicing_score(
            matched.notes,
            matched.root,
            matched.intervals,
            matched.accounted | matched.extras,
        )
        + bass_score(matched.notes, matched.root, matched.bass)
        + spelling.map_or(0, |spelling| root_context_penalty(matched.root, spelling))
        + dense_cluster_penalty(matched.notes, matched.root, matched.intervals, &matched.suffix)
        + tertian_support_score(matched.intervals, &matched.suffix)
        + incomplete_extension_penalty(matched.intervals, &matched.suffix)
        + suffix_complexity(&matched.suffix)
        + when(
            forced && !matched.notes.pitch_classes.contains(&matched.root),
            ABSENT_FORCED_ROOT,
        )
}

fn when(condition: bool, score: i32) -> i32 {
    if condition { score } else { 0 }
}

fn extra_cost(extras: u16, intervals: Intervals, suffix: &ChordSuffix) -> i32 {
    let add_count = suffix.rendered_metrics().add_count;

    extras.count_ones() as i32 * 8
        + when(suffix.has(Modifier::AddFlatThirteen), 24)
        + when(
            suffix.base == ChordBase::Five && !suffix.has(Modifier::AddFlatNine) && !suffix.modifiers.is_empty(),
            24,
        )
        + when(add_count > 1, (add_count - 1) * 18)
        + when(suffix.has_any(&[Modifier::AddMajorSeven, Modifier::AddFlatSeven]), 18)
        + when(
            suffix.has_any(&[Modifier::SharpNine, Modifier::AddSharpNine]) && !intervals.has(4),
            10,
        )
        + when(
            suffix.has_any(&[Modifier::FlatNine, Modifier::AddFlatNine]) && suffix.has(Modifier::AddNine),
            48,
        )
        + when(suffix.base.is_six() && !intervals.has(7), 22)
        + when(suffix.has(Modifier::AddSharpNine), 18)
        + when(suffix.base.is_six() && suffix.has(Modifier::AddEleven), 12)
        + when(
            suffix.base.is_sus() && suffix.has_any(&[Modifier::FlatNine, Modifier::AddFlatNine]) && !intervals.has(7),
            40,
        )
        + when(suffix.base.is_sus() && suffix.has_thirteenth() && !intervals.has(7), 16)
        + when(
            suffix.is_maj7()
                && suffix.has_any(&[Modifier::FlatThirteen, Modifier::AddFlatThirteen, Modifier::SharpFive]),
            72,
        )
        + when(
            suffix.base == ChordBase::MinorSeven
                && suffix.has_any(&[Modifier::FlatThirteen, Modifier::AddFlatThirteen])
                && !intervals.has(7),
            24,
        )
}

fn voicing_score(notes: &Notes, root: usize, intervals: Intervals, accounted: u16) -> i32 {
    let lowest_root = notes.lowest_root(root);
    let anchor = lowest_root.unwrap_or_else(|| implied_root_below(notes.bass_note, root));
    let root_height = lowest_root.map_or(12, |root_note| root_note.saturating_sub(notes.bass_note) as i32);
    let high_root = when(intervals.has(0) && root_height > 24, (root_height - 24) * 2);

    let low_colors = notes
        .notes
        .iter()
        .copied()
        .take_while(|note| *note < notes.bass_note + 12)
        .filter(|note| {
            let interval = (*note + 120 - anchor) % 12;
            matches!(interval, 1 | 2 | 5 | 9) && accounted & bit(interval) != 0
        })
        .count() as i32
        * 8;

    high_root + low_colors
}

fn bass_score(notes: &Notes, root: usize, bass: Option<usize>) -> i32 {
    let Some(bass) = bass else {
        return -8;
    };
    let interval = (bass + 12 - root) % 12;
    let slash = 5;
    let register = when(
        notes
            .lowest_root(root)
            .is_none_or(|root_note| notes.bass_note >= root_note),
        8,
    );

    slash
        + register
        + match interval {
            3 | 4 | 7 => 0,
            10 | 11 => 8,
            2 | 5 | 9 => 8,
            1 | 6 | 8 => 20,
            _ => 10,
        }
}

fn dense_cluster_penalty(notes: &Notes, root: usize, intervals: Intervals, suffix: &ChordSuffix) -> i32 {
    if notes.pitch_classes.len() < 6 || !notes.notes.windows(2).any(|notes| notes[1] == notes[0] + 1) {
        return 0;
    }

    let stable_dominant = intervals.has(4) && intervals.has(7) && intervals.has(10);
    let stable_major = suffix.is_maj7() && intervals.has(4) && intervals.has(7);
    let bass_leading_tone = notes.bass == (root + 11) % 12;

    when(!(stable_dominant || stable_major && bass_leading_tone), 30)
}

fn root_context_penalty(root: usize, spelling: &SpellingContext) -> i32 {
    let intervals = scale_intervals(spelling.scale);
    if intervals.len() != 7 {
        return 0;
    }

    let interval = (root + 12 - spelling.root_pitch_class) % 12;
    if intervals.contains(&interval) {
        0
    } else if matches!(interval, 1 | 3 | 6 | 8 | 10) {
        18
    } else {
        26
    }
}

fn tertian_support_score(intervals: Intervals, suffix: &ChordSuffix) -> i32 {
    if suffix.base.blocks_tertian_support() {
        return 0;
    }

    let complete_seventh =
        (intervals.has(3) || intervals.has(4)) && intervals.has(7) && (intervals.has(10) || intervals.has(11));
    when(complete_seventh, -16)
}

fn incomplete_extension_penalty(intervals: Intervals, suffix: &ChordSuffix) -> i32 {
    let sparse = intervals.count() < 5;
    when(sparse && suffix.has_thirteenth(), 24) + when(sparse && suffix.has_eleventh() && !suffix.base.is_sus(), 12)
}

fn suffix_complexity(suffix: &ChordSuffix) -> i32 {
    let metrics = suffix.rendered_metrics();
    metrics.len / 2
        + metrics.spaces * 5
        + metrics.add_count.min(1) * 4
        + when(suffix.base == ChordBase::Augmented && metrics.len > 3, 20)
}

fn implied_root_below(bass: usize, root: usize) -> usize {
    bass.saturating_sub((bass % 12 + 12 - root) % 12)
}
