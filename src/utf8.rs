// All bits in the following arrays represent the high bits of the bytes:
const UTF8_1_BYTES: [u8; 1] = [ 0b0                       ]; // 00000-0007F
const UTF8_2_BYTES: [u8; 2] = [ 0b110,   0b10             ]; // 00080-007FF
const UTF8_3_BYTES: [u8; 3] = [ 0b1110,  0b10, 0b10       ]; // 00800-0FFFF
const UTF8_4_BYTES: [u8; 4] = [ 0b11110, 0b10, 0b10, 0b10 ]; // 10000-1FFFF
/// The maximum length of bytes per character.
pub const MAX_UTF8_BYTES: usize = 4;

#[derive(PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum Unicode {
    Unknown,
    /// 0-127
    Range00000_0007F,
    /// 128-2047
    Range00080_007FF,
    /// 2048-65535
    Range00800_0FFFF,
    /// 65536-1114111
    Range10000_1FFFF,
}

pub struct Value<'a>(&'a [u8], Unicode);

impl<'a> Value<'a> {
    #[inline]
    pub fn unicode(&self) -> Unicode {
        self.1
    }
    
    #[inline]
    pub fn len(&self) -> usize {
        match self.1 {
            Unicode::Unknown => 0,
            Unicode::Range00000_0007F => 1,
            Unicode::Range00080_007FF => 2,
            Unicode::Range00800_0FFFF => 3,
            Unicode::Range10000_1FFFF => 4,
        }
    }

    #[inline]
    pub fn prefix(&self) -> &'a [u8] {
        match self.1 {
            Unicode::Unknown | Unicode::Range00000_0007F => unreachable!(),
            // We can use `unsafe { v.get_unchecked(...) }`, because we know the length.
            Unicode::Range00080_007FF => unsafe { self.0.get_unchecked(0..1) },
            Unicode::Range00800_0FFFF => unsafe { self.0.get_unchecked(0..2) },
            Unicode::Range10000_1FFFF => unsafe { self.0.get_unchecked(0..3) },
        }
    }

    #[inline]
    pub fn char(&self) -> u8 {
        match self.1 {
            Unicode::Unknown | Unicode::Range00000_0007F => unreachable!(),
            // We can use `unsafe { *v.get_unchecked(...) }`, because we know the length.
            Unicode::Range00080_007FF => unsafe { *self.0.get_unchecked(1) },
            Unicode::Range00800_0FFFF => unsafe { *self.0.get_unchecked(2) },
            Unicode::Range10000_1FFFF => unsafe { *self.0.get_unchecked(3) },
        }
    }
}

impl<'a> From<&'a [u8]> for Value<'a> {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        let unicode = match value.len() {
            4.. if is_4_bytes(value) => Unicode::Range10000_1FFFF,
            3.. if is_3_bytes(value) => Unicode::Range00800_0FFFF,
            2.. if is_2_bytes(value) => Unicode::Range00080_007FF,
            1.. if is_1_bytes(value) => Unicode::Range00000_0007F,
            _ => Unicode::Unknown,
        };
        Self(value, unicode)
    }
}

#[inline(always)]
const fn is_1_bytes(values: &[u8]) -> bool {
    (values[0] >> 7) == UTF8_1_BYTES[0]
}

#[inline(always)]
const fn is_2_bytes(values: &[u8]) -> bool {
    (values[0] >> 5) == UTF8_2_BYTES[0] && 
    (values[1] >> 6) == UTF8_2_BYTES[1]
}

#[inline(always)]
const fn is_3_bytes(values: &[u8]) -> bool {
    (values[0] >> 4) == UTF8_3_BYTES[0] && 
    (values[1] >> 6) == UTF8_3_BYTES[1] && 
    (values[2] >> 6) == UTF8_3_BYTES[2]
}

#[inline(always)]
const fn is_4_bytes(values: &[u8]) -> bool {
    (values[0] >> 3) == UTF8_4_BYTES[0] && 
    (values[1] >> 6) == UTF8_4_BYTES[1] && 
    (values[2] >> 6) == UTF8_4_BYTES[2] && 
    (values[3] >> 6) == UTF8_4_BYTES[3]
}
