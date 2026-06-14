mod detect;
mod intervals;
mod roman;
mod score;
mod symbol;
mod templates;

use crate::note::{Scale, SpellingContext};

use symbol::ChordSuffix;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub root: usize,
    pub bass: Option<usize>,
    suffix: ChordSuffix,
}

impl Chord {
    fn new(root: usize, bass: Option<usize>, suffix: ChordSuffix) -> Self {
        Self { root, bass, suffix }
    }

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

pub fn identify_chord_in_context(
    notes: &[usize],
    root_override: Option<usize>,
    spelling: &SpellingContext,
) -> Option<Chord> {
    detect::identify_chord_with_context(notes, root_override, Some(spelling))
}

pub fn format_chord(chord: &Chord, spelling: &SpellingContext) -> String {
    let mut name = format!("{}{}", spelling.harmonic_pitch_name(chord.root), chord.suffix());
    if let Some(bass) = chord.bass {
        name.push_str(" / ");
        name.push_str(&spelling.harmonic_pitch_name(bass));
    }
    name
}

pub fn roman_numeral(chord: &Chord, key_root: usize, scale: Scale) -> Option<String> {
    roman::roman_numeral(chord, key_root, scale)
}

#[cfg(test)]
mod tests {
    use crate::note::{RootNote, Scale, SpellingContext};

    use super::{format_chord, identify_chord_in_context, roman_numeral};

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

    fn roman_name(notes: &[usize], root: Option<usize>, spelling: &SpellingContext) -> String {
        identify_chord_in_context(notes, root, spelling)
            .and_then(|chord| roman_numeral(&chord, spelling.root_pitch_class, spelling.scale))
            .unwrap_or_else(|| "—".to_owned())
    }
}
