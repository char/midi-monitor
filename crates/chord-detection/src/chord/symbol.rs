use std::fmt::{self, Display, Formatter};

use super::intervals::{has_fifth, has_major_third, has_seventh};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordBase {
    Major,
    Minor,
    Five,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
    SixSus4,
    Six,
    MinorSix,
    SixNine,
    MinorSixNine,
    Seven,
    MajorSeven,
    MinorSeven,
    MinorMajorSeven,
    HalfDiminishedSeven,
    DiminishedSeven,
    Nine,
    MajorNine,
    MinorNine,
    Eleven,
    MinorEleven,
    Thirteen,
    ThirteenFlatNine,
    ThirteenSharpNine,
    MajorThirteen,
    MinorThirteen,
    SevenSus4,
    NineSus4,
    ThirteenSus4,
}

impl ChordBase {
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Major => "",
            Self::Minor => "m",
            Self::Five => "5",
            Self::Diminished => "dim",
            Self::Augmented => "aug",
            Self::Sus2 => "sus2",
            Self::Sus4 => "sus4",
            Self::SixSus4 => "6sus4",
            Self::Six => "6",
            Self::MinorSix => "m6",
            Self::SixNine => "6/9",
            Self::MinorSixNine => "m6/9",
            Self::Seven => "7",
            Self::MajorSeven => "maj7",
            Self::MinorSeven => "m7",
            Self::MinorMajorSeven => "mMaj7",
            Self::HalfDiminishedSeven => "m7b5",
            Self::DiminishedSeven => "dim7",
            Self::Nine => "9",
            Self::MajorNine => "maj9",
            Self::MinorNine => "m9",
            Self::Eleven => "11",
            Self::MinorEleven => "m11",
            Self::Thirteen => "13",
            Self::ThirteenFlatNine => "13b9",
            Self::ThirteenSharpNine => "13#9",
            Self::MajorThirteen => "maj13",
            Self::MinorThirteen => "m13",
            Self::SevenSus4 => "7sus4",
            Self::NineSus4 => "9sus4",
            Self::ThirteenSus4 => "13sus4",
        }
    }

    pub fn is_maj7(self) -> bool {
        matches!(self, Self::MajorSeven | Self::MajorNine | Self::MajorThirteen)
    }

    pub fn is_sus(self) -> bool {
        matches!(
            self,
            Self::Sus2 | Self::Sus4 | Self::SixSus4 | Self::SevenSus4 | Self::NineSus4 | Self::ThirteenSus4
        )
    }

    pub fn is_six(self) -> bool {
        matches!(
            self,
            Self::Six | Self::MinorSix | Self::SixNine | Self::MinorSixNine | Self::SixSus4
        )
    }

    pub fn is_minor(self) -> bool {
        matches!(
            self,
            Self::Minor
                | Self::Diminished
                | Self::MinorSix
                | Self::MinorSixNine
                | Self::MinorSeven
                | Self::MinorMajorSeven
                | Self::HalfDiminishedSeven
                | Self::DiminishedSeven
                | Self::MinorNine
                | Self::MinorEleven
                | Self::MinorThirteen
        )
    }

    pub fn is_diminished(self) -> bool {
        matches!(
            self,
            Self::Diminished | Self::HalfDiminishedSeven | Self::DiminishedSeven
        )
    }

    pub fn blocks_tertian_support(self) -> bool {
        self.is_sus() || matches!(self, Self::Diminished | Self::DiminishedSeven | Self::Augmented)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modifier {
    FlatNine,
    SharpNine,
    AddFlatNine,
    AddNine,
    AddSharpNine,
    Eleven,
    AddEleven,
    FlatFive,
    SharpEleven,
    AddSharpEleven,
    SharpFive,
    FlatThirteen,
    AddFlatThirteen,
    Thirteen,
    AddThirteen,
    Seven,
    MajorSeven,
    AddFlatSeven,
    AddMajorSeven,
}

impl Modifier {
    fn label(self) -> &'static str {
        match self {
            Self::FlatNine => "b9",
            Self::SharpNine => "#9",
            Self::AddFlatNine => "addb9",
            Self::AddNine => "add9",
            Self::AddSharpNine => "add#9",
            Self::Eleven => "11",
            Self::AddEleven => "add11",
            Self::FlatFive => "b5",
            Self::SharpEleven => "#11",
            Self::AddSharpEleven => "add#11",
            Self::SharpFive => "#5",
            Self::FlatThirteen => "b13",
            Self::AddFlatThirteen => "addb13",
            Self::Thirteen => "13",
            Self::AddThirteen => "add13",
            Self::Seven => "7",
            Self::MajorSeven => "maj7",
            Self::AddFlatSeven => "addb7",
            Self::AddMajorSeven => "addmaj7",
        }
    }

    fn label_without_add(self) -> &'static str {
        match self {
            Self::AddFlatNine => "b9",
            Self::AddNine => "9",
            Self::AddSharpNine => "#9",
            Self::AddEleven => "11",
            Self::AddSharpEleven => "#11",
            Self::AddFlatThirteen => "b13",
            Self::AddThirteen => "13",
            Self::AddFlatSeven => "b7",
            Self::AddMajorSeven => "maj7",
            _ => self.label(),
        }
    }

    fn is_add(self) -> bool {
        matches!(
            self,
            Self::AddFlatNine
                | Self::AddNine
                | Self::AddSharpNine
                | Self::AddEleven
                | Self::AddSharpEleven
                | Self::AddFlatThirteen
                | Self::AddThirteen
                | Self::AddFlatSeven
                | Self::AddMajorSeven
        )
    }

    fn rank(self) -> usize {
        match self {
            Self::FlatNine | Self::SharpNine | Self::AddFlatNine | Self::AddNine | Self::AddSharpNine => 0,
            Self::Eleven | Self::AddEleven | Self::FlatFive | Self::SharpEleven | Self::AddSharpEleven => 1,
            Self::SharpFive | Self::FlatThirteen | Self::AddFlatThirteen | Self::Thirteen | Self::AddThirteen => 2,
            Self::Seven | Self::MajorSeven | Self::AddFlatSeven | Self::AddMajorSeven => 3,
        }
    }

    pub fn is_eleventh(self) -> bool {
        matches!(
            self,
            Self::Eleven | Self::AddEleven | Self::SharpEleven | Self::AddSharpEleven
        )
    }

    pub fn is_thirteenth(self) -> bool {
        matches!(
            self,
            Self::SharpFive | Self::FlatThirteen | Self::AddFlatThirteen | Self::Thirteen | Self::AddThirteen
        )
    }
}

fn modifier_labels(modifiers: &[Modifier]) -> Vec<&'static str> {
    let strip_add = strip_add_modifier_prefixes(modifiers);
    modifiers
        .iter()
        .map(|modifier| {
            if strip_add {
                modifier.label_without_add()
            } else {
                modifier.label()
            }
        })
        .collect()
}

fn strip_add_modifier_prefixes(modifiers: &[Modifier]) -> bool {
    modifiers.iter().filter(|modifier| modifier.is_add()).count() > 1
}

#[derive(Clone, Copy)]
pub struct SuffixMetrics {
    pub len: i32,
    pub spaces: i32,
    pub add_count: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChordSuffix {
    pub base: ChordBase,
    pub modifiers: Vec<Modifier>,
}

impl ChordSuffix {
    pub fn bare(base: ChordBase) -> Self {
        Self {
            base,
            modifiers: Vec::new(),
        }
    }

    fn new(base: ChordBase, mut modifiers: Vec<Modifier>) -> Option<Self> {
        modifiers.sort_by_key(|modifier| modifier.rank());
        modifiers.dedup();

        if invalid_seventh_modifiers(&modifiers) {
            return None;
        }

        Some(Self { base, modifiers })
    }

    pub fn has(&self, modifier: Modifier) -> bool {
        self.modifiers.contains(&modifier)
    }

    pub fn has_any(&self, modifiers: &[Modifier]) -> bool {
        modifiers.iter().any(|modifier| self.has(*modifier))
    }

    pub fn has_eleventh(&self) -> bool {
        matches!(self.base, ChordBase::Eleven | ChordBase::MinorEleven)
            || self.modifiers.iter().any(|modifier| modifier.is_eleventh())
    }

    pub fn has_thirteenth(&self) -> bool {
        matches!(
            self.base,
            ChordBase::Thirteen
                | ChordBase::ThirteenFlatNine
                | ChordBase::ThirteenSharpNine
                | ChordBase::MajorThirteen
                | ChordBase::MinorThirteen
                | ChordBase::ThirteenSus4
        ) || self.modifiers.iter().any(|modifier| modifier.is_thirteenth())
    }

    pub fn is_maj7(&self) -> bool {
        self.base.is_maj7() || self.has(Modifier::MajorSeven)
    }

    pub fn rendered_metrics(&self) -> SuffixMetrics {
        let suffix = self.render();
        SuffixMetrics {
            len: suffix.len() as i32,
            spaces: suffix.bytes().filter(|byte| *byte == b' ').count() as i32,
            add_count: suffix.matches("add").count() as i32,
        }
    }

    fn renders_single_modifier_inline(&self) -> bool {
        if self.modifiers.len() != 1 {
            return false;
        }
        if self.base == ChordBase::Major {
            return matches!(self.modifiers[0], Modifier::Seven | Modifier::MajorSeven);
        }

        !self.modifiers[0].is_add()
            && !self.base.is_sus()
            && !matches!(self.modifiers[0], Modifier::Eleven | Modifier::Thirteen)
    }

    pub fn render(&self) -> String {
        canonical_suffix_rendering(self.render_without_normalization())
    }

    fn render_without_normalization(&self) -> String {
        let mut suffix = self.base.suffix().to_owned();
        if self.modifiers.is_empty() {
            return suffix;
        }

        let labels = modifier_labels(&self.modifiers);

        if self.renders_single_modifier_inline() {
            suffix.push_str(labels[0]);
        } else {
            suffix.push('(');
            suffix.push_str(&labels.join(" "));
            suffix.push(')');
        }

        suffix
    }
}

fn invalid_seventh_modifiers(modifiers: &[Modifier]) -> bool {
    let strip_add = strip_add_modifier_prefixes(modifiers);
    let has_minor_seventh = modifiers.contains(&Modifier::Seven);
    let has_major_seventh =
        modifiers.contains(&Modifier::MajorSeven) || strip_add && modifiers.contains(&Modifier::AddMajorSeven);
    let has_ungrouped_added_seventh = !strip_add
        && modifiers
            .iter()
            .any(|modifier| matches!(modifier, Modifier::AddFlatSeven | Modifier::AddMajorSeven));

    has_minor_seventh && has_major_seventh || has_ungrouped_added_seventh
}

impl Display for ChordSuffix {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

pub fn suffix_with_modifiers(base: ChordBase, extras: u16, accounted: u16) -> Option<ChordSuffix> {
    let context = ModifierContext::new(base, accounted);
    let mut modifiers = Vec::new();

    for interval in 1..12 {
        if extras & (1 << interval) != 0 {
            modifiers.push(modifier_for_extra(interval, context)?);
        }
    }

    ChordSuffix::new(base, modifiers)
}

#[derive(Clone, Copy)]
struct ModifierContext {
    has_chord_seventh: bool,
    has_major_third: bool,
    has_fifth: bool,
    base_is_sus: bool,
    base: ChordBase,
}

impl ModifierContext {
    fn new(base: ChordBase, accounted: u16) -> Self {
        Self {
            has_chord_seventh: has_seventh(accounted) || base == ChordBase::DiminishedSeven,
            has_major_third: has_major_third(accounted),
            has_fifth: has_fifth(accounted),
            base_is_sus: base.is_sus(),
            base,
        }
    }
}

fn modifier_for_extra(interval: usize, context: ModifierContext) -> Option<Modifier> {
    Some(match interval {
        1 => {
            if context.has_chord_seventh {
                Modifier::FlatNine
            } else {
                Modifier::AddFlatNine
            }
        }
        2 => Modifier::AddNine,
        3 => {
            if context.has_major_third && context.has_chord_seventh {
                Modifier::SharpNine
            } else {
                Modifier::AddSharpNine
            }
        }
        5 => {
            if context.has_chord_seventh && !context.base_is_sus {
                Modifier::Eleven
            } else {
                Modifier::AddEleven
            }
        }
        6 => {
            if !context.has_fifth && context.has_chord_seventh {
                Modifier::FlatFive
            } else if context.has_chord_seventh {
                Modifier::SharpEleven
            } else {
                Modifier::AddSharpEleven
            }
        }
        8 => {
            if !context.has_fifth && context.has_major_third {
                Modifier::SharpFive
            } else if context.has_chord_seventh {
                Modifier::FlatThirteen
            } else {
                Modifier::AddFlatThirteen
            }
        }
        9 => {
            if context.has_chord_seventh {
                Modifier::Thirteen
            } else {
                Modifier::AddThirteen
            }
        }
        10 => {
            if matches!(context.base, ChordBase::Major | ChordBase::Minor) {
                Modifier::Seven
            } else {
                Modifier::AddFlatSeven
            }
        }
        11 => {
            if context.base == ChordBase::Major {
                Modifier::MajorSeven
            } else {
                Modifier::AddMajorSeven
            }
        }
        _ => return None,
    })
}

fn canonical_suffix_rendering(suffix: String) -> String {
    match suffix.as_str() {
        "m7(11)" | "m(11 9)" | "m(9 11)" => "m11".to_owned(),
        "maj7(13)" => "maj13".to_owned(),
        "mMaj7(13)" => "mMaj13".to_owned(),
        "6sus4(addb9)" => "6sus4(b9)".to_owned(),
        "sus2(11 b13)" => "sus4(add9 b13)".to_owned(),
        _ => suffix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chord::intervals::bits;

    #[test]
    fn dominant_extras_are_extensions() {
        let context = ModifierContext::new(ChordBase::Seven, bits(&[0, 4, 7, 10]));

        assert_eq!(modifier_for_extra(1, context), Some(Modifier::FlatNine));
        assert_eq!(modifier_for_extra(3, context), Some(Modifier::SharpNine));
        assert_eq!(modifier_for_extra(6, context), Some(Modifier::SharpEleven));
        assert_eq!(modifier_for_extra(9, context), Some(Modifier::Thirteen));
    }

    #[test]
    fn triad_extras_are_additions() {
        let context = ModifierContext::new(ChordBase::Major, bits(&[0, 4, 7]));

        assert_eq!(modifier_for_extra(1, context), Some(Modifier::AddFlatNine));
        assert_eq!(modifier_for_extra(5, context), Some(Modifier::AddEleven));
        assert_eq!(modifier_for_extra(9, context), Some(Modifier::AddThirteen));
    }
}
