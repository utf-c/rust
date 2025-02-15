#[cfg(feature = "simd")]
mod simd;

/// This function uses SIMD (if the feature is enabled, otherwise a normal loop is used) to find a non-ASCII character 
/// and returns `true` if one is found, otherwise `false`.
/// 
/// # Example
/// ```
/// use utf_c::helper::contains_non_ascii;
/// let text = "Hello Wörld";
/// let result = contains_non_ascii(text);
/// assert_eq!(result, true);
/// ```
#[inline]
pub fn contains_non_ascii<T>(haystack: T) -> bool
where 
    T: AsRef<[u8]>,
{
    find_pos_byte_idx(haystack.as_ref()).is_some()
}

/// This function uses SIMD (if the feature is enabled, otherwise a normal loop is used) to find a non-ASCII character
/// and returns its index if one is found, otherwise `None`.
/// 
/// # Example
/// ```
/// use utf_c::helper::find_non_ascii_idx;
/// let text = "Hello Wörld";
/// let result = find_non_ascii_idx(text);
/// assert_eq!(result, Some(7));
/// ```
#[inline]
pub fn find_non_ascii_idx<T>(haystack: T) -> Option<usize>
where 
    T: AsRef<[u8]>,
{
    find_pos_byte_idx(haystack.as_ref())
}

/// This function uses SIMD (if the feature is enabled, otherwise a normal loop is used) to find a byte with the sign bit set to `1`.
/// 
/// Returns the index of the first byte that has the sign bit set to `1` (value greater than 127), otherwise `None`.
pub(crate) fn find_pos_byte_idx(bytes: &[u8]) -> Option<usize> {
    #[cfg(feature = "simd")]
    return unsafe { simd::find_pos_byte_idx(bytes) };

    #[cfg(not(feature = "simd"))]
    return bytes.iter().position(|b| test_sign_bit(*b));
}

/// Returns `true` if the sign bit is set, otherwise `false`.
#[inline(always)]
pub(crate) fn test_sign_bit(byte: u8) -> bool {
    // NOTE: ASCII characters have a value of 0-127, which means that the sign bit is never set.
    (byte & 0b10000000) != 0
}

#[cfg(test)]
mod tests {
    #[test]
    fn contains_non_ascii() {
        let test_cases: [(&[u8], bool); 13]  = [
                (&[ 0 ], false),
                (&[ 0, 0, 0 ], false),
                (&[ 128 ], true),
                (&[ 0, 0, 128 ], true),
                // SSE2 | NEON
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], true),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], true),
                // AVX2
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], true),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], true),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], true),
            ];

        for (idx, result) in test_cases.into_iter().enumerate() {
            let value = super::contains_non_ascii(result.0);
            assert_eq!(value, result.1, "failed at index {}", idx);
        }
    }

    #[test]
    fn find_pos_byte_idx() {
        let test_cases: [(&[u8], usize); 11] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 14),
            // SSE2 | NEON
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 15),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 16),
            // AVX2
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 32),
            // AVX2 + SSE2
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 9),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 34),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 48),
        ];
        
        for (idx, result) in test_cases.into_iter().enumerate() {
            let value = super::find_pos_byte_idx(result.0);
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }
}
