#[cfg(target_arch = "x86")]
use std::arch::x86;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as x86;

#[cfg(not(any(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"),
)))]
compile_error!("The current arch is not supported. Please disable the \"simd\" feature for utf-c, to get the best possible experience.");

#[cfg(not(any(
    // x86 | x86_64
    all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        any(target_feature = "avx2", target_feature = "sse2"),
    ), 
    // arm | aarch64 | arm64ec
    all(
        any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"), 
        target_feature = "neon",
    ),
)))]
compile_error!("The required features for SIMD are not available. Please disable the \"simd\" feature for utf-c, to get the best possible experience.");

pub unsafe fn next_non_ascii_pos(haystack: &[u8]) -> Option<usize> {
    const NEEDLE: u8 = 0b10000000;

    let mut remainder = 0;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_feature = "avx2")]
        // The following code will only be added if the feature is supported at compile time, otherwise this code will be executed slowly.
        if haystack.len() >= 32 {
            // Here we check during runtime whether this feature is supported by the currently used processor.
            if is_x86_feature_detected!("avx2") {
                let block_count = haystack.len() / 32;
                remainder = 32 * block_count;

                // Create a SIMD vector filled with the pattern
                let pattern_vec = x86::_mm256_set1_epi8(NEEDLE as i8);

                for i in (0..remainder).step_by(32) {
                    let chunk = haystack.as_ptr().add(i);

                    // Load the current chunk into a SIMD vector
                    let chunk_vec = x86::_mm256_loadu_si256(chunk as *const x86::__m256i);

                    // Compare each byte in the chunk with the pattern
                    let cmp_result = x86::_mm256_and_si256(chunk_vec, pattern_vec);

                    // Check if any byte matches
                    let mask = x86::_mm256_movemask_epi8(cmp_result);
                    if mask != 0 {
                        // We have found a non-ASCII character
                        let pos = i + (mask.trailing_zeros() as usize);
                        return Some(pos);
                    }
                }
            }
        }
        
        #[cfg(target_feature = "sse2")]
        // The following code will only be added if the feature is supported at compile time, otherwise this code will be executed slowly.
        if remainder == 0 && haystack.len() >= 16 {
            // Here we check during runtime whether this feature is supported by the currently used processor.
            if is_x86_feature_detected!("sse2") {
                let block_count = haystack.len() / 16;
                remainder = 16 * block_count;

                // Create a SIMD vector filled with the pattern
                let pattern_vec = x86::_mm_set1_epi8(NEEDLE as i8);

                for i in (0..remainder).step_by(16) {
                    let chunk = haystack.as_ptr().add(i);

                    // Load the current chunk into a SIMD vector
                    let chunk_vec = x86::_mm_loadu_si128(chunk as *const x86::__m128i);

                    // Compare each byte in the chunk with the pattern
                    let cmp_result = x86::_mm_and_si128(chunk_vec, pattern_vec);

                    // Check if any byte matches
                    let mask = x86::_mm_movemask_epi8(cmp_result);
                    if mask != 0 {
                        // We have found a non-ASCII character
                        let pos = i + (mask.trailing_zeros() as usize);
                        return Some(pos);
                    }
                }
            }
        }
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"))]
    {
        // TODO
    }

    haystack.iter().skip(remainder).position(|b| *b & NEEDLE != 0)
}
