use crate::note::{Scale, scale_intervals, semitone_delta};

use super::Chord;

pub fn roman_numeral(chord: &Chord, key_root: usize, scale: Scale) -> Option<String> {
    let mut out = roman_degree(chord.root, key_root, scale, chord.is_minor())?;
    let suffix = chord.suffix();
    out.push_str(&roman_suffix(&suffix, chord.is_minor(), chord.is_diminished()));
    if let Some(bass) = chord.bass {
        out.push_str(" / ");
        out.push_str(&roman_degree(bass, key_root, scale, false)?);
    }
    Some(out)
}

fn roman_degree(pitch_class: usize, key_root: usize, scale: Scale, minor: bool) -> Option<String> {
    let pitch_class = pitch_class % 12;
    let key_root = key_root % 12;
    let intervals = scale_intervals(scale);
    if intervals.len() != 7 {
        return None;
    }

    let (degree, accidental) = intervals
        .iter()
        .enumerate()
        .map(|(degree, interval)| {
            let scale_pitch_class = (key_root + interval) % 12;
            (degree, semitone_delta(scale_pitch_class, pitch_class))
        })
        .filter(|(_, accidental)| accidental.abs() <= 2)
        .min_by_key(|(degree, accidental)| {
            let tie_break = i32::from(*degree != 3 && *accidental > 0);
            (accidental.abs(), tie_break)
        })?;

    let numeral = ["I", "II", "III", "IV", "V", "VI", "VII"][degree];
    let accidental_prefix = if accidental < 0 { 'b' } else { '#' }
        .to_string()
        .repeat(accidental.unsigned_abs().into());

    Some(format!(
        "{}{}",
        accidental_prefix,
        if minor {
            numeral.to_lowercase()
        } else {
            numeral.to_owned()
        }
    ))
}

fn roman_suffix(suffix: &str, minor: bool, diminished: bool) -> String {
    let suffix = concise_roman_suffix(suffix);

    if diminished {
        if let Some(rest) = suffix.strip_prefix("m7b5") {
            return format!("ø7{rest}");
        }
        if let Some(rest) = suffix.strip_prefix("dim") {
            return format!("°{rest}");
        }
        return format!("°{suffix}");
    }

    suffix
        .strip_prefix(if minor { "m" } else { "" })
        .unwrap_or(&suffix)
        .to_owned()
}

fn concise_roman_suffix(suffix: &str) -> String {
    let Some(open) = suffix.find('(') else {
        return suffix.to_owned();
    };
    let Some(close) = suffix[open..].find(')').map(|close| open + close) else {
        return suffix.to_owned();
    };

    if suffix[open + 1..close].contains(' ') {
        format!("{}{}", &suffix[..open], &suffix[close + 1..])
    } else {
        suffix.to_owned()
    }
}
