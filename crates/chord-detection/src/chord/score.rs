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
    ScoreBreakdown::new(matched, forced, spelling).total()
}

struct ScoreBreakdown {
    template: i32,
    missing_root: i32,
    missing_essentials: i32,
    sparse_missing: i32,
    missing_seventh: i32,
    present_essentials: i32,
    optional_support: i32,
    extras: i32,
    voicing: i32,
    bass: i32,
    context: i32,
    density: i32,
    tertian_support: i32,
    incomplete_extension: i32,
    complexity: i32,
    absent_forced_root: i32,
}

impl ScoreBreakdown {
    fn new(matched: &FormulaMatch<'_>, forced: bool, spelling: Option<&SpellingContext>) -> Self {
        let missing_essentials = matched.missing_essentials().count_ones() as i32;

        Self {
            template: matched.template.rank,
            missing_root: i32::from(matched.missing_required & bit(0) != 0)
                * if forced {
                    MISSING_FORCED_ROOT
                } else {
                    MISSING_INFERRED_ROOT
                },
            missing_essentials: missing_essentials * MISSING_ESSENTIAL,
            sparse_missing: i32::from(matched.notes.pitch_classes.len() <= 2)
                * missing_essentials
                * MISSING_ESSENTIAL_IN_SPARSE_CHORD,
            missing_seventh: i32::from(
                has_seventh(matched.template.required) && matched.missing_required & (bit(10) | bit(11)) != 0,
            ) * MISSING_SEVENTH,
            present_essentials: (matched.present_required & !bit(0)).count_ones() as i32 * PRESENT_ESSENTIAL_BONUS,
            optional_support: matched.present_optional.count_ones() as i32 * PRESENT_OPTIONAL_BONUS
                + matched.missing_optional.count_ones() as i32 * MISSING_OPTIONAL,
            extras: extra_cost(matched.extras, matched.intervals, &matched.suffix),
            voicing: voicing_score(
                matched.notes,
                matched.root,
                matched.intervals,
                matched.accounted | matched.extras,
            ),
            bass: bass_score(matched.notes, matched.root, matched.bass),
            context: spelling.map_or(0, |spelling| root_context_penalty(matched.root, spelling)),
            density: dense_cluster_penalty(matched.notes, matched.root, matched.intervals, &matched.suffix),
            tertian_support: tertian_support_score(matched.intervals, &matched.suffix),
            incomplete_extension: incomplete_extension_penalty(matched.intervals, &matched.suffix),
            complexity: suffix_complexity(&matched.suffix),
            absent_forced_root: i32::from(forced && !matched.notes.pitch_classes.contains(&matched.root))
                * ABSENT_FORCED_ROOT,
        }
    }

    fn total(&self) -> i32 {
        self.template
            + self.missing_root
            + self.missing_essentials
            + self.sparse_missing
            + self.missing_seventh
            + self.present_essentials
            + self.optional_support
            + self.extras
            + self.voicing
            + self.bass
            + self.context
            + self.density
            + self.tertian_support
            + self.incomplete_extension
            + self.complexity
            + self.absent_forced_root
    }
}

fn extra_cost(extras: u16, intervals: Intervals, suffix: &ChordSuffix) -> i32 {
    let mut cost = extras.count_ones() as i32 * 8;
    if suffix.has(Modifier::AddFlatThirteen) {
        cost += 24;
    }
    if suffix.base == ChordBase::Five && !suffix.has(Modifier::AddFlatNine) && !suffix.modifiers.is_empty() {
        cost += 24;
    }
    let add_count = suffix.rendered_metrics().add_count;
    if add_count > 1 {
        cost += (add_count - 1) * 18;
    }
    if suffix.has_any(&[Modifier::AddMajorSeven, Modifier::AddFlatSeven]) {
        cost += 18;
    }
    if suffix.has_any(&[Modifier::SharpNine, Modifier::AddSharpNine]) && !intervals.has(4) {
        cost += 10;
    }
    if suffix.has_any(&[Modifier::FlatNine, Modifier::AddFlatNine]) && suffix.has(Modifier::AddNine) {
        cost += 48;
    }
    if suffix.base.is_six() && !intervals.has(7) {
        cost += 22;
    }
    if suffix.has(Modifier::AddSharpNine) {
        cost += 18;
    }
    if suffix.base.is_six() && suffix.has(Modifier::AddEleven) {
        cost += 12;
    }
    if suffix.base.is_sus() && suffix.has_any(&[Modifier::FlatNine, Modifier::AddFlatNine]) && !intervals.has(7) {
        cost += 40;
    }
    if suffix.base.is_sus() && suffix.has_thirteenth() && !intervals.has(7) {
        cost += 16;
    }
    if suffix.is_maj7() && suffix.has_any(&[Modifier::FlatThirteen, Modifier::AddFlatThirteen, Modifier::SharpFive]) {
        cost += 72;
    }
    if suffix.base == ChordBase::MinorSeven
        && suffix.has_any(&[Modifier::FlatThirteen, Modifier::AddFlatThirteen])
        && !intervals.has(7)
    {
        cost += 24;
    }
    cost
}

fn voicing_score(notes: &Notes, root: usize, intervals: Intervals, accounted: u16) -> i32 {
    let anchor = notes
        .lowest_root(root)
        .unwrap_or_else(|| implied_root_below(notes.bass_note, root));

    let root_height = notes
        .lowest_root(root)
        .map_or(12, |root_note| root_note.saturating_sub(notes.bass_note) as i32);
    let high_root = if intervals.has(0) && root_height > 24 {
        (root_height - 24) * 2
    } else {
        0
    };

    let low_colors = notes
        .notes
        .iter()
        .copied()
        .take_while(|note| *note < notes.bass_note + 12)
        .filter(|note| {
            let interval = (*note + 120 - anchor) % 12;
            is_color_interval(interval) && accounted & bit(interval) != 0
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
    let register = if notes
        .lowest_root(root)
        .is_some_and(|root_note| notes.bass_note < root_note)
    {
        0
    } else {
        8
    };

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

    if stable_dominant || stable_major && bass_leading_tone {
        0
    } else {
        30
    }
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
    if complete_seventh { -16 } else { 0 }
}

fn incomplete_extension_penalty(intervals: Intervals, suffix: &ChordSuffix) -> i32 {
    let sparse = intervals.count() < 5;
    i32::from(sparse && suffix.has_thirteenth()) * 24
        + i32::from(sparse && suffix.has_eleventh() && !suffix.base.is_sus()) * 12
}

fn suffix_complexity(suffix: &ChordSuffix) -> i32 {
    let metrics = suffix.rendered_metrics();
    metrics.len / 2
        + metrics.spaces * 5
        + metrics.add_count.min(1) * 4
        + i32::from(suffix.base == ChordBase::Augmented && metrics.len > 3) * 20
}

fn implied_root_below(bass: usize, root: usize) -> usize {
    let bass = bass as isize;
    let mut note = root as isize;
    while note + 12 <= bass {
        note += 12;
    }
    while note > bass {
        note -= 12;
    }
    note.max(0) as usize
}

fn is_color_interval(interval: usize) -> bool {
    matches!(interval, 1 | 2 | 5 | 9)
}
