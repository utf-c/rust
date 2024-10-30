const C_UNICODE_00000_0007F: u8 = 0b0; // ASCII
const C_UNICODE_00080_007FF: [u8; 2] = [0b110, 0b10];
const C_UNICODE_00800_0FFFF: [u8; 3] = [0b1110, 0b10, 0b10];
const C_UNICODE_10000_1FFFF: [u8; 4] = [0b11110, 0b10, 0b10, 0b10];
/// The maximum length of bytes per character.
pub const C_MAX_LEN: usize = 4;

#[derive(PartialEq)]
#[repr(i8)]
pub enum CharSetType<'a> {
    Unknown,
    Unicode00000_0007f,
    Unicode00080_007ff(&'a [u8]),
    Unicode00800_0ffff(&'a [u8]),
    Unicode10000_1ffff(&'a [u8]),
}

impl<'a> CharSetType<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Unknown => 0,
            Self::Unicode00000_0007f => 1,
            Self::Unicode00080_007ff(_) => 2,
            Self::Unicode00800_0ffff(_) => 3,
            Self::Unicode10000_1ffff(_) => 4,
        } 
    }

    #[inline]
    pub fn set(&self) -> &'a [u8] {
        match *self {
            Self::Unknown | Self::Unicode00000_0007f => unreachable!(),
            Self::Unicode00080_007ff(v) => &v[0..1],
            Self::Unicode00800_0ffff(v) => &v[0..2],
            Self::Unicode10000_1ffff(v) => &v[0..3],
        }
    }

    #[inline]
    pub fn value(self) -> u8 {
        match self {
            Self::Unknown | Self::Unicode00000_0007f => unreachable!(),
            Self::Unicode00080_007ff(v) => v[1],
            Self::Unicode00800_0ffff(v) => v[2],
            Self::Unicode10000_1ffff(v) => v[3],
        }
    }
}

impl<'a> From<&'a [u8]> for CharSetType<'a> {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        match value.len() {
            4.. if is_unicode10000_1ffff(value) => Self::Unicode10000_1ffff(&value[0..=3]),
            3.. if is_unicode00800_0ffff(value) => Self::Unicode00800_0ffff(&value[0..=2]),
            2.. if is_unicode00080_007ff(value) => Self::Unicode00080_007ff(&value[0..=1]),
            1.. if is_unicode00000_0007f(value) => Self::Unicode00000_0007f,
            _ => Self::Unknown 
        }
    }
}

#[inline(always)]
fn is_unicode00000_0007f(values: &[u8]) -> bool {
    (values[0] >> 7) == C_UNICODE_00000_0007F
}

#[inline(always)]
fn is_unicode00080_007ff(values: &[u8]) -> bool {
    (values[0] >> 5) == C_UNICODE_00080_007FF[0] && 
    (values[1] >> 6) == C_UNICODE_00080_007FF[1]
}

#[inline(always)]
fn is_unicode00800_0ffff(values: &[u8]) -> bool {
    (values[0] >> 4) == C_UNICODE_00800_0FFFF[0] && 
    (values[1] >> 6) == C_UNICODE_00800_0FFFF[1] && 
    (values[2] >> 6) == C_UNICODE_00800_0FFFF[2]
}

#[inline(always)]
fn is_unicode10000_1ffff(values: &[u8]) -> bool {
    (values[0] >> 3) == C_UNICODE_10000_1FFFF[0] && 
    (values[1] >> 6) == C_UNICODE_10000_1FFFF[1] && 
    (values[2] >> 6) == C_UNICODE_10000_1FFFF[2] && 
    (values[3] >> 6) == C_UNICODE_10000_1FFFF[3]
}
