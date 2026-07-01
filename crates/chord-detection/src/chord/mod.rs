mod detect;
mod intervals;
mod roman;
mod score;
mod symbol;
mod templates;

use crate::note::{Scale, SpellingContext, scale_intervals};

use symbol::{ChordBase, ChordSuffix, Modifier};

const BASS_NOTE_CUTOFF: usize = 60; // middle C
const CHORD_SCALE_CANDIDATES: &[Scale] = &[
    Scale::Major,
    Scale::Minor,
    Scale::HarmonicMinor,
    Scale::MelodicMinor,
    Scale::Dorian,
    Scale::Phrygian,
    Scale::Lydian,
    Scale::Mixolydian,
    Scale::Locrian,
    Scale::PhrygianDominant,
];
const CHORD_SCALE_FALLBACK_CANDIDATES: &[Scale] = &[
    Scale::HarmonicMajor,
    Scale::LydianAugmented,
    Scale::Altered,
    Scale::WholeTone,
    Scale::HalfWholeDiminished,
    Scale::WholeHalfDiminished,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Analysis {
    Chord(Chord),
    UnknownChord(usize),
    Interval(PlayedInterval),
    Note(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayedInterval {
    root: usize,
    semitones: usize,
}

impl PlayedInterval {
    fn between(a: usize, b: usize) -> Self {
        Self {
            root: a.min(b) % 12,
            semitones: a.abs_diff(b),
        }
    }

    pub fn name(&self, spelling: &SpellingContext) -> String {
        format!(
            "{} {}",
            spelling.scale_pitch_name(self.root),
            interval_description(self.semitones)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub root: usize,
    pub bass: Option<usize>,
    suffix: ChordSuffix,
    pitch_classes: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChordScale {
    pub root: usize,
    pub scale: Scale,
}

impl Chord {
    pub fn suffix(&self) -> String {
        self.suffix.to_string()
    }

    pub fn is_minor(&self) -> bool {
        self.suffix.base.is_minor()
    }

    pub fn is_diminished(&self) -> bool {
        self.suffix.base.is_diminished()
    }
}

pub fn identify_analysis_in_context(
    notes: &[usize],
    root_override: Option<usize>,
    spelling: &SpellingContext,
) -> Option<Analysis> {
    let mut notes = notes.to_vec();
    notes.sort_unstable();

    match notes.as_slice() {
        [] => None,
        [note] => identify_single_note(*note, root_override, spelling),
        [a, b] => Some(Analysis::Interval(PlayedInterval::between(*a, *b))),
        _ => detect::identify_chord_with_context(&notes, root_override, Some(spelling)).map(Analysis::Chord),
    }
}

pub fn format_analysis(analysis: &Analysis, spelling: &SpellingContext) -> String {
    match analysis {
        Analysis::Chord(chord) => format_chord(chord, spelling),
        Analysis::UnknownChord(root) => format!("{} ?", spelling.scale_pitch_name(*root)),
        Analysis::Interval(interval) => interval.name(spelling),
        Analysis::Note(pitch_class) => spelling.scale_pitch_name(*pitch_class),
    }
}

pub fn format_chord(chord: &Chord, spelling: &SpellingContext) -> String {
    let mut name = format!("{}{}", spelling.scale_pitch_name(chord.root), chord.suffix());
    if let Some(bass) = chord.bass {
        name.push_str(" / ");
        name.push_str(&spelling.scale_pitch_name(bass));
    }
    name
}

pub fn chord_scales(chord: &Chord) -> Vec<ChordScale> {
    if chord.pitch_classes.len() <= 1 {
        return Vec::new();
    }

    let root = chord.bass.unwrap_or(chord.root);
    let preferred = preferred_chord_scale(chord).filter(|scale| scale.root == root);
    let mut scales = matching_chord_scales(chord, root, CHORD_SCALE_CANDIDATES);
    prune_minor_variants(&mut scales);
    if scales.is_empty() {
        scales = matching_chord_scales(chord, root, CHORD_SCALE_FALLBACK_CANDIDATES);
    }

    if let Some(preferred) = preferred
        && let Some(index) = scales.iter().position(|scale| *scale == preferred)
    {
        scales.remove(index);
        let plain_count = scales
            .iter()
            .take_while(|scale| matches!(scale.scale, Scale::Major | Scale::Minor))
            .count();
        scales.insert(plain_count, preferred);
    }

    scales
}

pub fn format_chord_scales(chord_scales: &[ChordScale], spelling: &SpellingContext) -> Option<String> {
    let first = chord_scales.first()?;
    if chord_scales.iter().all(|scale| scale.root == first.root) {
        let scales = chord_scales
            .iter()
            .map(|scale| scale_abbreviation(scale.scale))
            .collect::<Vec<_>>()
            .join("/");
        return Some(format!("{} {scales}", spelling.scale_pitch_name(first.root)));
    }

    Some(
        chord_scales
            .iter()
            .map(|scale| {
                format!(
                    "{} {}",
                    spelling.scale_pitch_name(scale.root),
                    scale_abbreviation(scale.scale)
                )
            })
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn scale_abbreviation(scale: Scale) -> &'static str {
    match scale {
        Scale::Chromatic => "chrom",
        Scale::Major => "maj",
        Scale::Minor => "min",
        Scale::HarmonicMinor => "hmin",
        Scale::MelodicMinor => "mel min",
        Scale::HarmonicMajor => "hmaj",
        Scale::Dorian => "dor",
        Scale::Phrygian => "phr",
        Scale::Lydian => "lyd",
        Scale::LydianAugmented => "lyd aug",
        Scale::Mixolydian => "mix",
        Scale::Locrian => "loc",
        Scale::Altered => "altered",
        Scale::PhrygianDominant => "phr dom",
        Scale::WholeTone => "whole tone",
        Scale::HalfWholeDiminished => "half-whole dim",
        Scale::WholeHalfDiminished => "whole-half dim",
    }
}

pub fn roman_numeral(chord: &Chord, key_root: usize, scale: Scale) -> Option<String> {
    roman::roman_numeral(chord, key_root, scale)
}

fn preferred_chord_scale(chord: &Chord) -> Option<ChordScale> {
    if let Some(scale) = slash_chord_scale(chord) {
        return Some(scale);
    }

    Some(ChordScale {
        root: chord.root,
        scale: suffix_preferred_scale(&chord.suffix)?,
    })
}

fn slash_chord_scale(chord: &Chord) -> Option<ChordScale> {
    let bass = chord.bass?;
    let bass_interval = (bass + 12 - chord.root) % 12;

    (bass_interval == 10
        && dominant_with_major_third(&chord.suffix)
        && chord_tones_fit_scale(chord, bass, Scale::Lydian))
    .then_some(ChordScale {
        root: bass,
        scale: Scale::Lydian,
    })
}

fn suffix_preferred_scale(suffix: &ChordSuffix) -> Option<Scale> {
    use ChordBase::*;

    let scale = if major_family(suffix) && suffix.has_any(&[Modifier::SharpEleven, Modifier::AddSharpEleven]) {
        Scale::Lydian
    } else if major_family(suffix) && suffix.has_any(&[Modifier::Eleven, Modifier::AddEleven]) {
        Scale::Major
    } else if dominant_with_major_third(suffix) {
        Scale::Mixolydian
    } else {
        match suffix.base {
            Major | Six | SixNine => Scale::Major,
            MajorSeven | MajorNine | MajorThirteen => Scale::Lydian,
            Minor => Scale::Minor,
            MinorSix | MinorSixNine | MinorSeven | MinorNine | MinorEleven | MinorThirteen => Scale::Dorian,
            MinorMajorSeven => Scale::HarmonicMinor,
            Sus2 | Sus4 | SixSus4 | SevenSus4 | NineSus4 | ThirteenSus4 => Scale::Mixolydian,
            Diminished | HalfDiminishedSeven => Scale::Locrian,
            Five | Augmented | DiminishedSeven | ThirteenFlatNine | ThirteenSharpNine => return None,
            Eleven | Seven | Nine | Thirteen => unreachable!(),
        }
    };

    Some(scale)
}

fn matching_chord_scales(chord: &Chord, root: usize, candidates: &[Scale]) -> Vec<ChordScale> {
    candidates
        .iter()
        .copied()
        .filter(|scale| chord_tones_fit_scale(chord, root, *scale) && scale_has_enough_context(chord, root, *scale))
        .map(|scale| ChordScale { root, scale })
        .collect()
}

fn prune_minor_variants(scales: &mut Vec<ChordScale>) {
    if scales.iter().any(|scale| scale.scale == Scale::Minor) {
        scales.retain(|scale| !matches!(scale.scale, Scale::HarmonicMinor | Scale::MelodicMinor));
    }
}

fn scale_has_enough_context(chord: &Chord, root: usize, scale: Scale) -> bool {
    match scale {
        Scale::HarmonicMajor => chord.suffix.base != ChordBase::Augmented,
        Scale::HalfWholeDiminished => {
            chord_has_interval(chord, root, 10)
                && (chord_has_interval(chord, root, 1) || chord_has_interval(chord, root, 3))
        }
        _ => true,
    }
}

fn chord_has_interval(chord: &Chord, root: usize, interval: usize) -> bool {
    chord
        .pitch_classes
        .iter()
        .any(|pitch_class| (pitch_class + 12 - root) % 12 == interval)
}

fn chord_tones_fit_scale(chord: &Chord, root: usize, scale: Scale) -> bool {
    let intervals = scale_intervals(scale);
    chord
        .pitch_classes
        .iter()
        .all(|pitch_class| intervals.contains(&((pitch_class + 12 - root) % 12)))
}

fn major_family(suffix: &ChordSuffix) -> bool {
    matches!(
        suffix.base,
        ChordBase::Major
            | ChordBase::Six
            | ChordBase::SixNine
            | ChordBase::MajorSeven
            | ChordBase::MajorNine
            | ChordBase::MajorThirteen
    )
}

fn dominant_with_major_third(suffix: &ChordSuffix) -> bool {
    matches!(
        suffix.base,
        ChordBase::Seven | ChordBase::Nine | ChordBase::Eleven | ChordBase::Thirteen
    )
}

fn identify_single_note(note: usize, root_override: Option<usize>, spelling: &SpellingContext) -> Option<Analysis> {
    if let Some(chord) = detect::identify_chord_with_context(&[note], root_override, Some(spelling)) {
        return Some(Analysis::Chord(chord));
    }

    if note >= BASS_NOTE_CUTOFF {
        Some(Analysis::Note(note % 12))
    } else {
        Some(Analysis::UnknownChord(root_override.unwrap_or(note % 12)))
    }
}

fn interval_description(semitones: usize) -> String {
    let octaves = semitones / 12;
    let simple = semitones % 12;

    if simple == 0 {
        return match octaves {
            0 => "unison".to_owned(),
            1 => "octave".to_owned(),
            2 => "double octave".to_owned(),
            3 => "triple octave".to_owned(),
            _ => format!("{octaves} octaves"),
        };
    }

    let names = match simple {
        1 => &["minor second", "minor ninth"][..],
        2 => &["major second", "major ninth"][..],
        3 => &["minor third", "minor tenth"][..],
        4 => &["major third", "major tenth"][..],
        5 => &["perfect fourth", "perfect eleventh"][..],
        6 => &["tritone"][..],
        7 => &["perfect fifth", "perfect twelfth"][..],
        8 => &["minor sixth", "minor thirteenth"][..],
        9 => &["major sixth", "major thirteenth"][..],
        10 => &["minor seventh"][..],
        11 => &["major seventh"][..],
        _ => unreachable!(),
    };

    if let Some(name) = names.get(octaves) {
        return (*name).to_owned();
    }

    let octave_label = if octaves == 1 { "octave" } else { "octaves" };
    format!("{} + {octaves} {octave_label}", names[0])
}

#[cfg(test)]
mod tests {
    use crate::note::{RootNote, Scale, SpellingContext};

    use super::{
        chord_scales, detect, format_analysis, format_chord, format_chord_scales, identify_analysis_in_context,
        roman_numeral,
    };

    #[test]
    fn identifies_representative_chords() {
        let chromatic = SpellingContext::new(RootNote::C, Scale::Chromatic);
        for (expected, notes, root) in [
            ("C", &[60, 64, 67][..], None),
            ("Cm", &[60, 63, 67], None),
            ("Cdim", &[60, 63, 66], None),
            ("Caug", &[60, 64, 68], None),
            ("C9", &[60, 62, 64, 67, 70], None),
            ("C13b9", &[48, 52, 55, 58, 61, 69], None),
            ("G#maj7 / C", &[36, 44, 51, 55, 60, 63], None),
            ("C / E", &[64, 67, 72], Some(0)),
            ("C9sus4 / F", &[53, 55, 58, 60, 62], Some(0)),
        ] {
            assert_eq!(chord_name(notes, root, &chromatic), expected);
        }

        let g_sharp_minor = SpellingContext::new(RootNote::GSharp, Scale::Minor);
        for (expected, notes, root) in [
            ("Emaj7", &[52, 56, 59, 63][..], None),
            ("C#m7b5 / G", &[55, 59, 61], None),
            ("G13sus4", &[43, 50, 53, 57, 60, 64, 69], None),
            ("B6", &[35, 47, 63, 68, 75, 87], None),
            ("F#6sus4(b9)", &[30, 42, 67, 71, 73, 87], None),
        ] {
            assert_eq!(chord_name(notes, root, &g_sharp_minor), expected);
        }
    }

    #[test]
    fn labels_single_notes_and_intervals() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);

        assert_eq!(analysis_name(&[62], None, &c_major), "Dm");
        assert_eq!(analysis_name(&[49], None, &c_major), "Db ?");
        assert_eq!(analysis_name(&[61], None, &c_major), "Db");
        assert_eq!(analysis_name(&[60, 64], None, &c_major), "C major third");
        assert_eq!(analysis_name(&[60, 67], None, &c_major), "C perfect fifth");
        assert_eq!(analysis_name(&[60, 72], None, &c_major), "C octave");
        assert_eq!(analysis_name(&[60, 76], None, &c_major), "C major tenth");
        assert_eq!(analysis_name(&[60, 81], None, &c_major), "C major thirteenth");
        assert_eq!(analysis_name(&[60, 82], None, &c_major), "C minor seventh + 1 octave");
        assert_eq!(analysis_name(&[60, 112], None, &c_major), "C major third + 4 octaves");
    }

    #[test]
    fn identifies_representative_roman_numerals() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);
        for (expected, notes, root) in [
            ("I", &[48, 55, 60, 64, 67][..], None),
            ("vi7", &[45, 57, 60, 64, 67, 69], None),
            ("V7sus4 / II", &[38, 53, 55, 60, 62, 67], None),
            ("ii7 / IV", &[41, 53, 60, 62, 65], None),
        ] {
            assert_eq!(roman_name(notes, root, &c_major), expected);
        }
    }

    #[test]
    fn suggests_representative_chord_scales() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);
        for (expected, notes, root) in [
            ("C mix/dor", &[48, 55, 58, 62, 65, 69][..], None),
            ("Eb maj/lyd", &[51, 55, 58, 62, 65], None),
            ("Bb lyd", &[46, 48, 52, 53, 55], None),
        ] {
            assert_eq!(chord_scale_names(notes, root, &c_major), expected);
        }
        assert_eq!(chord_name(&[46, 48, 52, 53, 55], None, &c_major), "C7(11) / Bb");

        let g_flat_major = SpellingContext::new(RootNote::GFlat, Scale::Major);
        assert_eq!(
            chord_scale_names(&[54, 58, 61, 63, 65, 68], None, &g_flat_major),
            "Gb maj/lyd"
        );
    }

    #[test]
    fn suggests_harmonic_minor_family_scales() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);

        assert_eq!(
            chord_scale_names(&[48, 52, 55, 58, 61, 68], None, &c_major),
            "C phr dom"
        );
        assert_eq!(chord_scale_names(&[48, 51, 55, 59, 68], None, &c_major), "C hmin");
    }

    #[test]
    fn omits_harmonic_and_melodic_minor_when_plain_minor_fits() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);

        assert_eq!(chord_scale_names(&[48, 51, 55], None, &c_major), "C min/dor/phr");
    }

    #[test]
    fn falls_back_to_extended_chord_scale_colours() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);

        assert_eq!(
            chord_scale_names(&[48, 52, 58, 63, 66, 68], None, &c_major),
            "C altered"
        );
        assert_eq!(
            chord_scale_names(&[48, 52, 58, 63, 66], None, &c_major),
            "C altered/half-whole dim"
        );
        assert_eq!(chord_scale_names(&[48, 51, 54, 57], None, &c_major), "C whole-half dim");
        assert_eq!(
            chord_scale_names(&[48, 52, 56, 62], None, &c_major),
            "C lyd aug/whole tone"
        );
        assert_eq!(chord_scale_names(&[47, 51, 54, 57, 65], None, &c_major), "—");
    }

    #[test]
    fn omits_chord_scales_for_single_pitch_classes() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);

        assert_eq!(chord_scale_names(&[60], None, &c_major), "—");
        assert_eq!(chord_scale_names(&[48, 60, 72], None, &c_major), "—");
    }

    fn chord_name(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        detect::identify_chord_with_context(notes, root, Some(spelling))
            .map(|chord| format_chord(&chord, spelling))
            .unwrap_or_else(|| "—".to_owned())
    }

    fn analysis_name(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        identify_analysis_in_context(notes, root, spelling)
            .map(|analysis| format_analysis(&analysis, spelling))
            .unwrap_or_else(|| "—".to_owned())
    }

    fn roman_name(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        detect::identify_chord_with_context(notes, root, Some(spelling))
            .and_then(|chord| roman_numeral(&chord, spelling.root_pitch_class, spelling.scale))
            .unwrap_or_else(|| "—".to_owned())
    }

    fn chord_scale_names(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        detect::identify_chord_with_context(notes, root, Some(spelling))
            .and_then(|chord| format_chord_scales(&chord_scales(&chord), spelling))
            .unwrap_or_else(|| "—".to_owned())
    }
}
