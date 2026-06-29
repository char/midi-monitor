use midi_monitor_chord_detection::{RootNote, Scale};
use truce::prelude::*;

#[derive(ParamEnum)]
pub enum RootNoteParam {
    C,
    #[name = "C#"]
    CSharp,
    #[name = "Db"]
    DFlat,
    D,
    #[name = "D#"]
    DSharp,
    #[name = "Eb"]
    EFlat,
    E,
    F,
    #[name = "F#"]
    FSharp,
    #[name = "Gb"]
    GFlat,
    G,
    #[name = "G#"]
    GSharp,
    #[name = "Ab"]
    AFlat,
    A,
    #[name = "A#"]
    ASharp,
    #[name = "Bb"]
    BFlat,
    B,
}

#[derive(ParamEnum)]
pub enum ScaleParam {
    Chromatic,
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    #[name = "Harmonic Minor"]
    HarmonicMinor,
    #[name = "Phrygian Dominant"]
    PhrygianDominant,
}

#[derive(ParamEnum)]
pub enum ChordRootParam {
    None,
    C,
    #[name = "C#"]
    CSharp,
    D,
    #[name = "D#"]
    DSharp,
    E,
    F,
    #[name = "F#"]
    FSharp,
    G,
    #[name = "G#"]
    GSharp,
    A,
    #[name = "A#"]
    ASharp,
    B,
}

impl From<RootNoteParam> for RootNote {
    fn from(value: RootNoteParam) -> Self {
        match value {
            RootNoteParam::C => Self::C,
            RootNoteParam::CSharp => Self::CSharp,
            RootNoteParam::DFlat => Self::DFlat,
            RootNoteParam::D => Self::D,
            RootNoteParam::DSharp => Self::DSharp,
            RootNoteParam::EFlat => Self::EFlat,
            RootNoteParam::E => Self::E,
            RootNoteParam::F => Self::F,
            RootNoteParam::FSharp => Self::FSharp,
            RootNoteParam::GFlat => Self::GFlat,
            RootNoteParam::G => Self::G,
            RootNoteParam::GSharp => Self::GSharp,
            RootNoteParam::AFlat => Self::AFlat,
            RootNoteParam::A => Self::A,
            RootNoteParam::ASharp => Self::ASharp,
            RootNoteParam::BFlat => Self::BFlat,
            RootNoteParam::B => Self::B,
        }
    }
}

impl From<ScaleParam> for Scale {
    fn from(value: ScaleParam) -> Self {
        match value {
            ScaleParam::Chromatic => Self::Chromatic,
            ScaleParam::Major => Self::Major,
            ScaleParam::Minor => Self::Minor,
            ScaleParam::Dorian => Self::Dorian,
            ScaleParam::Phrygian => Self::Phrygian,
            ScaleParam::Lydian => Self::Lydian,
            ScaleParam::Mixolydian => Self::Mixolydian,
            ScaleParam::Locrian => Self::Locrian,
            ScaleParam::HarmonicMinor => Self::HarmonicMinor,
            ScaleParam::PhrygianDominant => Self::PhrygianDominant,
        }
    }
}

impl ChordRootParam {
    pub fn pitch_class(self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::C => Some(0),
            Self::CSharp => Some(1),
            Self::D => Some(2),
            Self::DSharp => Some(3),
            Self::E => Some(4),
            Self::F => Some(5),
            Self::FSharp => Some(6),
            Self::G => Some(7),
            Self::GSharp => Some(8),
            Self::A => Some(9),
            Self::ASharp => Some(10),
            Self::B => Some(11),
        }
    }
}
