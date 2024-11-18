#[cfg(target_arch = "x86")]
use std::arch::x86;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as x86;
#[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
use core::arch::aarch64 as arm;

#[cfg(not(any(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_arch = "aarch64", target_arch = "arm64ec"),
)))]
compile_error!("The current arch is not supported. Please disable the \"simd\" feature for utf-c, to get the best possible experience.");

#[cfg(not(any(
    // x86 | x86_64
    all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        any(target_feature = "avx2", target_feature = "sse2"),
    ), 
    // aarch64 | arm64ec
    all(
        any(target_arch = "aarch64", target_arch = "arm64ec"), 
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

    #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
    {
        #[cfg(target_feature = "neon")]
        // The following code will only be added if the feature is supported at compile time, otherwise this code will be executed slowly.
        if haystack.len() >= 16 {
            // Here we check during runtime whether this feature is supported by the currently used processor.
            if std::arch::is_aarch64_feature_detected!("neon") {
                let block_count = haystack.len() / 16;
                remainder = 16 * block_count;
    
                // Create a SIMD vector filled with the pattern.
                let pattern_vec = arm::vdupq_n_u8(NEEDLE);
    
                for i in (0..remainder).step_by(16) {
                    let chunk = haystack.as_ptr().add(i);
    
                    // Load the current chunk into a SIMD vector.
                    let chunk_vec = arm::vld1q_u8(chunk);
    
                    // Compare each byte in the chunk with the pattern.
                    let cmp_result = arm::vandq_u8(chunk_vec, pattern_vec);
    
                    // Check if any byte matches.
                    let mask = neon_movemask_epu8(cmp_result);
                    if mask != 0 {
                        // We have found a non-ASCII character.
                        let pos = i + (mask.trailing_zeros() as usize);
                        return Some(pos);
                    }
                }
            }
        }
    }

    haystack.iter().skip(remainder).position(|b| *b & NEEDLE != 0)
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
#[cfg(target_feature = "neon")]
/// An alternative to `_mm_movemask_epi8` (SSE2) for NEON.
unsafe fn neon_movemask_epu8(value: arm::uint8x16_t) -> u16 {
    // Shift each u8 element in `value` 7 bits to the right (preserving the sign bit)
    // and then reinterpret `shift_u8` (vector of 16x-u8) as a vector of 8x-u16.
    let shift_u8 = arm::vshrq_n_u8(value, 7);
    let vec_8x16 = arm::vreinterpretq_u16_u8(shift_u8);

    // Shift (and accumulate) each u16 element in `vec_8x16` 7 bits to the right 
    // and then reinterpret `shift_u16` (vector of 8x-u16) as a vector of 4x-u32.
    let shift_u16 = arm::vsraq_n_u16(vec_8x16, vec_8x16, 7);
    let vec_4x32 = arm::vreinterpretq_u32_u16(shift_u16);

    // Shift (and accumulate) each u32 element in `vec_4x32` 14 bits to the right 
    // and then reinterpret `shift_u32` (vector of 4x-u32) as a vector of 2x-u64.
    let shift_u32 = arm::vsraq_n_u32(vec_4x32, vec_4x32, 14);
    let vec_2x64 = arm::vreinterpretq_u64_u32(shift_u32);

    // Shift (and accumulate) each u64 element in `vec_2x64` 28 bits to the right 
    // and then reinterpret `shift_u64` (vector of 2x-u64) as a vector of 16x-u8.
    let shift_u64 = arm::vsraq_n_u64(vec_2x64, vec_2x64, 28);
    let vec_16x8 = arm::vreinterpretq_u8_u64(shift_u64);
    // Our result is now in the first and ninth element of `vec_16x8` as "low" and "high".

    // Get our "low" and "high" element from `vec_16x8` as u8 data type.
    let (low, high): (u8, u8) = (
        arm::vgetq_lane_u8(vec_16x8, 0),
        arm::vgetq_lane_u8(vec_16x8, 8),
    );
    
    // Our final result:
    ((high as u16) << 8) | (low as u16)
}
