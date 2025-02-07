#[cfg(feature = "simd")]
mod simd;

/// This function checks whether the haystack consists only of ASCII characters.
pub fn only_ascii<T>(haystack: T) -> bool
where 
    T: AsRef<[u8]>,
{
    #[cfg(feature = "simd")]
    return unsafe { simd::next_non_ascii_pos(haystack.as_ref()) }.is_none();

    #[cfg(not(feature = "simd"))]
    return haystack.as_ref().iter().all(|b| *b & 0b10000000 == 0);
}

/// This function returns the index of the next non-ASCII character.
pub fn next_non_ascii_idx<T>(haystack: T) -> Option<usize>
where 
    T: AsRef<[u8]>,
{
    #[cfg(feature = "simd")]
    return unsafe { simd::next_non_ascii_pos(haystack.as_ref()) };

    #[cfg(not(feature = "simd"))]
    return haystack.as_ref().iter().position(|b| *b & 0b10000000 != 0);
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_ascii() {
        const RESULTS: [(&[u8], bool); 16]  = [
                (&[ 0 ], true),
                (&[ 0, 0, 0 ], true),
                (&[ 128 ], false),
                (&[ 0, 0, 128 ], false),
                // SSE2 | NEON
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], true),
                (&[ 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], true),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], false),
                // AVX2
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], true),
                (&[ 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], false),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], true),
                (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], false),
            ];

        for (idx, result) in RESULTS.into_iter().enumerate() {
            let value = super::only_ascii(result.0);
            assert_eq!(value, result.1, "failed at index {}", idx);
        }
    }

    #[test]
    fn next_non_ascii_idx() {
        const RESULTS: [(&[u8], usize); 11] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 14),
            // SSE2 | NEON
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 15),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 16),
            // AVX2
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 32),
            // AVX2 + SSE2
            (&[ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 128 ], 43),
            (&[ 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 47),
            (&[ 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 48),
        ];
        
        for (idx, result) in RESULTS.into_iter().enumerate() {
            let value = super::next_non_ascii_idx(result.0);
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }
}
