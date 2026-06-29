use crate::note::{SpellingContext, scale_intervals};

use super::{
    Chord,
    intervals::{Intervals, bit},
    score::score_candidate,
    symbol::{ChordSuffix, suffix_with_modifiers},
    templates::{TEMPLATES, Template, scale_triad},
};

pub struct Notes {
    pub notes: Vec<usize>,
    pub pitch_classes: Vec<usize>,
    pub bass_note: usize,
    pub bass: usize,
}

impl Notes {
    fn new(notes: &[usize]) -> Option<Self> {
        let mut notes = notes.to_vec();
        notes.sort_unstable();
        let bass_note = *notes.first()?;
        let bass = bass_note % 12;

        let mut pitch_classes = notes.iter().map(|note| note % 12).collect::<Vec<_>>();
        pitch_classes.sort_unstable();
        pitch_classes.dedup();

        Some(Self {
            notes,
            pitch_classes,
            bass_note,
            bass,
        })
    }

    pub fn lowest_root(&self, root: usize) -> Option<usize> {
        self.notes.iter().copied().find(|note| note % 12 == root)
    }

    fn roots(&self, forced_root: Option<usize>, spelling: Option<&SpellingContext>) -> Vec<usize> {
        if let Some(root) = forced_root {
            return vec![root % 12];
        }

        let mut roots = vec![self.bass];

        if let Some(spelling) = spelling {
            for interval in scale_intervals(spelling.scale) {
                let root = (spelling.root_pitch_class + interval) % 12;
                if self.pitch_classes.contains(&root) && !roots.contains(&root) {
                    roots.push(root);
                }
            }
        }

        for pitch_class in &self.pitch_classes {
            if !roots.contains(pitch_class) {
                roots.push(*pitch_class);
            }
        }

        roots
    }
}

pub struct FormulaMatch<'a> {
    pub notes: &'a Notes,
    pub root: usize,
    pub template: &'a Template,
    pub intervals: Intervals,
    pub present_required: u16,
    pub missing_required: u16,
    pub present_optional: u16,
    pub missing_optional: u16,
    pub accounted: u16,
    pub extras: u16,
    pub suffix: ChordSuffix,
    pub bass: Option<usize>,
}

impl<'a> FormulaMatch<'a> {
    fn new(notes: &'a Notes, root: usize, intervals: Intervals, template: &'a Template, forced: bool) -> Option<Self> {
        let accounted = template.required | template.optional;
        let present_required = intervals.present(template.required);
        let missing_required = intervals.missing(template.required);
        let present_optional = intervals.present(template.optional);
        let missing_optional = intervals.missing(template.optional);
        let missing_essentials = missing_required & !bit(0);

        if !forced && missing_essentials == template.required & !bit(0) {
            return None;
        }

        let extras = intervals.extras_after(accounted);
        let suffix = suffix_with_modifiers(template.base, extras, accounted)?;
        let bass = (notes.bass != root).then_some(notes.bass);

        Some(Self {
            notes,
            root,
            template,
            intervals,
            present_required,
            missing_required,
            present_optional,
            missing_optional,
            accounted,
            extras,
            suffix,
            bass,
        })
    }
}

pub fn identify_chord_with_context(
    notes: &[usize],
    root_override: Option<usize>,
    spelling: Option<&SpellingContext>,
) -> Option<Chord> {
    let notes = Notes::new(notes)?;
    let forced = root_override.map(|root| root % 12);
    if notes.pitch_classes.len() == 1 {
        return single_note_chord(&notes, forced, spelling);
    }

    notes
        .roots(forced, spelling)
        .into_iter()
        .flat_map(|root| candidates_for_root(&notes, root, forced.is_some(), spelling))
        .min_by_key(|(score, _)| *score)
        .map(|(_, chord)| chord)
}

fn single_note_chord(notes: &Notes, forced_root: Option<usize>, spelling: Option<&SpellingContext>) -> Option<Chord> {
    let root = forced_root.unwrap_or(notes.bass);
    spelling
        .and_then(|spelling| scale_triad(root, spelling))
        .map(|base| Chord {
            root,
            bass: (notes.bass != root).then_some(notes.bass),
            suffix: ChordSuffix::bare(base),
        })
}

fn candidates_for_root(
    notes: &Notes,
    root: usize,
    forced: bool,
    spelling: Option<&SpellingContext>,
) -> Vec<(i32, Chord)> {
    let intervals = Intervals::from_pitch_classes(notes.pitch_classes.iter().copied(), root);
    if !forced && !intervals.has(0) && notes.pitch_classes.len() > 1 {
        return Vec::new();
    }

    TEMPLATES
        .iter()
        .filter_map(|template| {
            let matched = FormulaMatch::new(notes, root, intervals, template, forced)?;
            let score = score_candidate(&matched, forced, spelling);

            Some((
                score,
                Chord {
                    root,
                    bass: matched.bass,
                    suffix: matched.suffix,
                },
            ))
        })
        .collect()
}
