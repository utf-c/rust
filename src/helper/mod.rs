#[cfg(feature="simd")]
mod simd;

/// This function checks whether the haystack consists only of ASCII characters.
pub fn only_ascii(haystack: &[u8]) -> bool {
    #[cfg(feature = "simd")]
    return unsafe { simd::next_non_ascii_pos(haystack) }.is_none();

    #[cfg(not(feature = "simd"))]
    return haystack.iter().all(|b| *b & 0b10000000 == 0);
}

pub(crate) fn next_non_ascii_pos(haystack: &[u8]) -> Option<usize> {
    #[cfg(feature = "simd")]
    return unsafe { simd::next_non_ascii_pos(haystack) };

    #[cfg(not(feature = "simd"))]
    return haystack.iter().position(|b| *b & 0b10000000 != 0);
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
    fn next_non_ascii_pos() {
        const RESULTS: [(&[u8], usize); 6] = [
            (&[0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 1, 2, 3, 128], 5),
            (&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 128], 14),
            // SSE2 | NEON
            (&[0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128], 5),
            (&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128], 15),
            // AVX2
            (&[0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128], 5),
            (&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128], 15),
        ];
        
        for (idx, result) in RESULTS.into_iter().enumerate() {
            let value = super::next_non_ascii_pos(result.0);
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }
}
