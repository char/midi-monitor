#[derive(Clone, Copy)]
pub struct Intervals(u16);

impl Intervals {
    pub fn from_pitch_classes(pitch_classes: impl IntoIterator<Item = usize>, root: usize) -> Self {
        debug_assert!(root < 12);
        let mut mask = 0;
        for pitch_class in pitch_classes {
            debug_assert!(pitch_class < 12);
            mask |= bit((pitch_class + 12 - root) % 12);
        }
        Self(mask)
    }

    pub fn has(self, interval: usize) -> bool {
        self.present(bit(interval)) != 0
    }

    pub fn count(self) -> i32 {
        self.0.count_ones() as i32
    }

    pub fn present(self, mask: u16) -> u16 {
        self.0 & mask
    }

    pub fn missing(self, mask: u16) -> u16 {
        mask & !self.0
    }

    pub fn extras_after(self, accounted: u16) -> u16 {
        self.0 & !accounted
    }
}

pub const fn bits(intervals: &[usize]) -> u16 {
    let mut out = 0;
    let mut i = 0;
    while i < intervals.len() {
        out |= bit(intervals[i]);
        i += 1;
    }
    out
}

pub const fn bit(interval: usize) -> u16 {
    debug_assert!(interval < 12);
    1 << interval
}

pub fn has_major_third(mask: u16) -> bool {
    mask & bit(4) != 0
}

pub fn has_fifth(mask: u16) -> bool {
    mask & (bit(6) | bit(7) | bit(8)) != 0
}

pub fn has_seventh(mask: u16) -> bool {
    mask & (bit(10) | bit(11)) != 0
}
