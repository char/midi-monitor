mod detect;
mod intervals;
mod roman;
mod score;
mod symbol;
mod templates;

use crate::note::{Scale, SpellingContext};

use symbol::ChordSuffix;

const BASS_NOTE_CUTOFF: usize = 60; // middle C

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
        _ => identify_chord_in_context(&notes, root_override, spelling).map(Analysis::Chord),
    }
}

pub fn identify_chord_in_context(
    notes: &[usize],
    root_override: Option<usize>,
    spelling: &SpellingContext,
) -> Option<Chord> {
    detect::identify_chord_with_context(notes, root_override, Some(spelling))
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

pub fn roman_numeral(chord: &Chord, key_root: usize, scale: Scale) -> Option<String> {
    roman::roman_numeral(chord, key_root, scale)
}

fn identify_single_note(note: usize, root_override: Option<usize>, spelling: &SpellingContext) -> Option<Analysis> {
    if let Some(chord) = identify_chord_in_context(&[note], root_override, spelling) {
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
        format_analysis, format_chord, identify_analysis_in_context, identify_chord_in_context, roman_numeral,
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

    fn chord_name(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        identify_chord_in_context(notes, root, spelling)
            .map(|chord| format_chord(&chord, spelling))
            .unwrap_or_else(|| "—".to_owned())
    }

    fn analysis_name(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        identify_analysis_in_context(notes, root, spelling)
            .map(|analysis| format_analysis(&analysis, spelling))
            .unwrap_or_else(|| "—".to_owned())
    }

    fn roman_name(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        identify_chord_in_context(notes, root, spelling)
            .and_then(|chord| roman_numeral(&chord, spelling.root_pitch_class, spelling.scale))
            .unwrap_or_else(|| "—".to_owned())
    }
}
