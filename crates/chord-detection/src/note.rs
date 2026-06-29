#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootNote {
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Chromatic,
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    HarmonicMinor,
    PhrygianDominant,
}

#[derive(Clone, Copy)]
struct NoteSpelling {
    letter: usize,
    accidental: i8,
}

impl NoteSpelling {
    fn for_pitch_class(letter: usize, pitch_class: usize) -> Self {
        debug_assert!(letter < 7);
        let accidental = semitone_delta(natural_pitch_class(letter), pitch_class % 12);
        Self { letter, accidental }
    }

    fn pitch_class(self) -> usize {
        (natural_pitch_class(self.letter) as i8 + self.accidental).rem_euclid(12) as usize
    }

    fn name(self) -> String {
        let letter = char::from(b"CDEFGAB"[self.letter]);
        let accidental = if self.accidental < 0 { 'b' } else { '#' };
        format!(
            "{letter}{}",
            accidental.to_string().repeat(self.accidental.unsigned_abs().into())
        )
    }

    #[cfg(test)]
    fn octave(self, note: usize) -> i32 {
        let spelled_offset = natural_pitch_class(self.letter) as i32 + i32::from(self.accidental);
        (note as i32 - spelled_offset).div_euclid(12) - 2
    }
}

#[derive(Clone)]
pub struct SpellingContext {
    pub root_pitch_class: usize,
    pub scale: Scale,
    root_prefers_flats: bool,
    pitch_classes: [Option<NoteSpelling>; 12],
}

impl SpellingContext {
    pub fn new(root: RootNote, scale: Scale) -> Self {
        let root_spelling = root.spelling();
        let root_pitch_class = root_spelling.pitch_class();
        let mut pitch_classes = [None; 12];
        pitch_classes[root_pitch_class] = Some(root_spelling);

        if let Some(letter_steps) = scale_letter_steps(scale) {
            for (interval, letter_step) in scale_intervals(scale).iter().zip(letter_steps) {
                let pitch_class = (root_pitch_class + interval) % 12;
                let letter = (root_spelling.letter + letter_step) % 7;
                pitch_classes[pitch_class] = Some(NoteSpelling::for_pitch_class(letter, pitch_class));
            }
        }

        let sharps = pitch_classes
            .iter()
            .flatten()
            .filter(|spelling| spelling.accidental > 0)
            .count();
        let flats = pitch_classes
            .iter()
            .flatten()
            .filter(|spelling| spelling.accidental < 0)
            .count();

        Self {
            root_pitch_class,
            scale,
            root_prefers_flats: flats > sharps || flats == sharps && root_spelling.accidental < 0,
            pitch_classes,
        }
    }

    pub fn pitch_name(&self, pitch_class: usize) -> String {
        self.spelling_for(pitch_class % 12).name()
    }

    pub fn scale_pitch_name(&self, pitch_class: usize) -> String {
        let pitch_class = pitch_class % 12;
        let intervals = scale_intervals(self.scale);
        let root_name = self.pitch_name(self.root_pitch_class);
        let Some(root_letter) = root_name
            .bytes()
            .next()
            .and_then(|letter| b"CDEFGAB".iter().position(|candidate| *candidate == letter))
        else {
            return self.pitch_name(pitch_class);
        };

        if intervals.len() != 7 {
            return self.pitch_name(pitch_class);
        }

        let Some((degree, accidental)) = intervals
            .iter()
            .enumerate()
            .map(|(degree, interval)| {
                let letter = (root_letter + degree) % 7;
                let scale_pitch_class = (self.root_pitch_class + interval) % 12;
                (
                    degree,
                    semitone_delta(natural_pitch_class(letter), pitch_class),
                    semitone_delta(scale_pitch_class, pitch_class),
                )
            })
            .filter(|(_, accidental, scale_delta)| accidental.abs() <= 2 && scale_delta.abs() <= 2)
            .min_by_key(|(degree, accidental, scale_delta)| {
                let tie_break = i32::from(*degree != 3 && *scale_delta > 0);
                (scale_delta.abs(), accidental.abs(), tie_break)
            })
            .map(|(degree, accidental, _)| (degree, accidental))
        else {
            return self.pitch_name(pitch_class);
        };

        NoteSpelling {
            letter: (root_letter + degree) % 7,
            accidental,
        }
        .name()
    }

    #[cfg(test)]
    pub fn note_name(&self, note: usize) -> String {
        let spelling = self.spelling_for(note % 12);
        format!("{}{}", spelling.name(), spelling.octave(note))
    }

    fn spelling_for(&self, pitch_class: usize) -> NoteSpelling {
        debug_assert!(pitch_class < 12);
        self.pitch_classes[pitch_class].unwrap_or_else(|| best_spelling(pitch_class, self.root_prefers_flats))
    }
}

impl RootNote {
    pub fn pitch_class(self) -> usize {
        self.spelling().pitch_class()
    }

    fn spelling(self) -> NoteSpelling {
        parse_spelling(match self {
            Self::C => "C",
            Self::CSharp => "C#",
            Self::DFlat => "Db",
            Self::D => "D",
            Self::DSharp => "D#",
            Self::EFlat => "Eb",
            Self::E => "E",
            Self::F => "F",
            Self::FSharp => "F#",
            Self::GFlat => "Gb",
            Self::G => "G",
            Self::GSharp => "G#",
            Self::AFlat => "Ab",
            Self::A => "A",
            Self::ASharp => "A#",
            Self::BFlat => "Bb",
            Self::B => "B",
        })
    }
}

fn best_spelling(pitch_class: usize, prefer_flats: bool) -> NoteSpelling {
    (0..7)
        .map(|letter| NoteSpelling::for_pitch_class(letter, pitch_class))
        .filter(|spelling| (-2..=2).contains(&spelling.accidental))
        .min_by_key(|spelling| {
            let wrong_direction = i32::from(
                spelling.accidental != 0
                    && (prefer_flats && spelling.accidental > 0 || !prefer_flats && spelling.accidental < 0),
            );
            (wrong_direction, spelling.accidental.abs(), spelling.letter)
        })
        .unwrap_or(NoteSpelling {
            letter: 0,
            accidental: 0,
        })
}

fn parse_spelling(name: &str) -> NoteSpelling {
    let mut chars = name.chars();
    let letter = chars.next().and_then(|ch| "CDEFGAB".find(ch)).unwrap_or(0);
    let accidental = chars.fold(0, |sum, ch| {
        sum + match ch {
            '#' => 1,
            'b' => -1,
            _ => 0,
        }
    });
    NoteSpelling { letter, accidental }
}

fn natural_pitch_class(letter: usize) -> usize {
    if letter < 3 { letter * 2 } else { letter * 2 - 1 }
}

pub fn scale_intervals(scale: Scale) -> &'static [usize] {
    match scale {
        Scale::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
        Scale::Minor => &[0, 2, 3, 5, 7, 8, 10],
        Scale::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
        Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
        Scale::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
        Scale::Lydian => &[0, 2, 4, 6, 7, 9, 11],
        Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
        Scale::Locrian => &[0, 1, 3, 5, 6, 8, 10],
        Scale::PhrygianDominant => &[0, 1, 4, 5, 7, 8, 10],
    }
}

fn scale_letter_steps(scale: Scale) -> Option<&'static [usize]> {
    match scale {
        Scale::Chromatic => None,
        _ => Some(&[0, 1, 2, 3, 4, 5, 6]),
    }
}

pub fn semitone_delta(from: usize, to: usize) -> i8 {
    let mut delta = to as i8 - from as i8;
    while delta > 6 {
        delta -= 12;
    }
    while delta < -6 {
        delta += 12;
    }
    delta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrygian_dominant_uses_flat_second_and_major_third() {
        assert_eq!(scale_intervals(Scale::PhrygianDominant), &[0, 1, 4, 5, 7, 8, 10]);

        let c = SpellingContext::new(RootNote::C, Scale::PhrygianDominant);
        assert_eq!(c.pitch_name(1), "Db");
        assert_eq!(c.pitch_name(4), "E");
    }

    #[test]
    fn sharp_major_keys_use_theoretical_spellings() {
        let c_sharp = SpellingContext::new(RootNote::CSharp, Scale::Major);
        let f_sharp = SpellingContext::new(RootNote::FSharp, Scale::Major);

        assert_eq!(c_sharp.pitch_name(0), "B#");
        assert_eq!(c_sharp.note_name(60), "B#2");
        assert_eq!(f_sharp.pitch_name(5), "E#");
        assert_eq!(f_sharp.note_name(65), "E#3");
    }

    #[test]
    fn flat_major_keys_use_theoretical_spellings() {
        let d_flat = SpellingContext::new(RootNote::DFlat, Scale::Major);
        let g_flat = SpellingContext::new(RootNote::GFlat, Scale::Major);

        assert_eq!(d_flat.pitch_name(1), "Db");
        assert_eq!(d_flat.pitch_name(0), "C");
        assert_eq!(g_flat.pitch_name(11), "Cb");
        assert_eq!(g_flat.note_name(71), "Cb4");
    }

    #[test]
    fn scale_pitch_names_use_contextual_accidentals() {
        let c_major = SpellingContext::new(RootNote::C, Scale::Major);
        assert_eq!(c_major.pitch_name(1), "C#");
        assert_eq!(c_major.scale_pitch_name(1), "Db");
        assert_eq!(c_major.scale_pitch_name(6), "F#");

        let c_minor = SpellingContext::new(RootNote::C, Scale::Minor);
        assert_eq!(c_minor.scale_pitch_name(4), "E");
        assert_eq!(c_minor.scale_pitch_name(9), "A");
        assert_eq!(c_minor.scale_pitch_name(11), "B");
    }
}
