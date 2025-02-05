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
    // x86
    all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        any(target_feature = "avx2", target_feature = "sse2"),
    ), 
    // arm
    all(
        any(target_arch = "aarch64", target_arch = "arm64ec"), 
        target_feature = "neon",
    ),
)))]
compile_error!("The required features for SIMD are not available. Please disable the \"simd\" feature for utf-c, to get the best possible experience.");

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"), 
    target_feature = "avx2"
))]
unsafe fn nnap_avx2(haystack: &[u8], remainder: &mut usize) -> Option<usize> {
    const BYTES_LEN: usize = 32;
    
    // We use this macro to check at runtime whether the CPU feature "avx2" is available.
    if is_x86_feature_detected!("avx2") {
        let block_count = haystack.len() / BYTES_LEN;

        let chunk_ptr = haystack.as_ptr();
        for block in 0..block_count {
            // Load the current chunk into a SIMD vector.
            let chunk_pos = BYTES_LEN * block;
            let simd_vec = x86::_mm256_loadu_si256(chunk_ptr.add(chunk_pos) as *const x86::__m256i);

            // Check if a byte with a sign bit was found.
            let mask = x86::_mm256_movemask_epi8(simd_vec);
            if mask != 0 {
                // We found a non-ASCII character at the following index:
                let idx = chunk_pos + (mask.trailing_zeros() as usize);
                return Some(idx);
            }
        }

        *remainder = block_count % BYTES_LEN;
    }
    None
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"), 
    target_feature = "sse2"
))]
unsafe fn nnap_sse2(haystack: &[u8], remainder: &mut usize) -> Option<usize> {
    const BYTES_LEN: usize = 16;

    // We use this macro to check at runtime whether the CPU feature "sse2" is available.
    if is_x86_feature_detected!("sse2") {
        let old_block_count = *remainder / BYTES_LEN;
        let block_count = haystack.len() / BYTES_LEN;

        let chunk_ptr = haystack.as_ptr();
        for block in old_block_count..block_count {
            // Load the current chunk into a SIMD vector.
            let chunk_pos = BYTES_LEN * block;
            let simd_vec = x86::_mm_loadu_si128(chunk_ptr.add(chunk_pos) as *const x86::__m128i);

            // Check if a byte with a sign bit was found.
            let mask = x86::_mm_movemask_epi8(simd_vec);
            if mask != 0 {
                // We found a non-ASCII character at the following index:
                let idx = chunk_pos + (mask.trailing_zeros() as usize);
                return Some(idx);
            }
        }

        *remainder = block_count % BYTES_LEN;
    }
    None
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm64ec"), 
    target_feature = "neon"
))]
unsafe fn nnap_neon(haystack: &[u8], remainder: &mut usize) -> Option<usize> {
    const BYTES_LEN: usize = 16;

    // We use this macro to check at runtime whether the CPU feature "neon" is available.
    if std::arch::is_aarch64_feature_detected!("neon") {
        let block_count = haystack.len() / BYTES_LEN;

        let chunk_ptr = haystack.as_ptr();
        for block in 0..block_count {
            // Load the current chunk into a SIMD vector.
            let chunk_pos = BYTES_LEN * block;
            let simd_vec = arm::vld1q_u8(chunk_ptr.add(chunk_pos));

            // Check if a byte with a sign bit was found.
            let mask = neon_movemask_epu8(simd_vec);
            if mask != 0 {
                // We found a non-ASCII character at the following index:
                let idx = chunk_pos + (mask.trailing_zeros() as usize);
                return Some(idx);
            }
        }

        *remainder = block_count % BYTES_LEN;
    }
    None
}

pub unsafe fn next_non_ascii_pos(haystack: &[u8]) -> Option<usize> {
    // The needle we are looking for.
    // We are looking for a byte where the sign bit is set.
    // (ASCII has a value of 0-127, which means that the sign bit is never set)
    const NEEDLE: u8 = 0b10000000;

    // At the end of this function, the number (remainder) of bytes are skipped.
    let mut remainder = 0;

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        target_feature = "avx2"
    ))]
    if haystack.len() >= 32 {
        let result = nnap_avx2(haystack, &mut remainder);
        if result.is_some() {
            return result;
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        target_feature = "sse2"
    ))]
    if (haystack.len() - remainder) >= 16 {
        let result = nnap_sse2(haystack, &mut remainder);
        if result.is_some() {
            return result;
        }
    }

    #[cfg(all(
        any(target_arch = "aarch64", target_arch = "arm64ec"), 
        target_feature = "neon"
    ))]
    if haystack.len() >= 16 {
        let result = nnap_neon(haystack, &mut remainder);
        if result.is_some() {
            return result;
        }
    }
    
    // Search the remaining bytes for the needle.
    haystack.iter()
        .skip(remainder)
        .position(|b| *b & NEEDLE != 0)
        .map(|result| remainder + result)
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm64ec"), 
    target_feature = "neon"
))]
/// An alternative to `_mm_movemask_epi8` (SSE2) for NEON.
/// 
/// Note: "big endian" support is based on a theory.
/// https://github.com/utf-c/neon_movemask_epu8/blob/main/README.md
unsafe fn neon_movemask_epu8(value: arm::uint8x16_t) -> u16 {
    // For the description below we have as an example a 16x-u8 vector where each element has the sign bit set:
    // [
    //   0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
    //   0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
    // ]
    
    // We shift the sign bit of all bytes to the right by (N=7).
    // 11111111
    // |______
    //        |
    // 00000001
    let shift_u8 = arm::vshrq_n_u8(value, 7);
    // We now turn the vector 16x-u8 into 8x-u16.
    // Here is an example of what happens to the elements:
    // 00000001(u8) + 
    // 00000001(u8) = 
    // 00000001 00000001(u16)
    let vec_8x16 = arm::vreinterpretq_u16_u8(shift_u8);

    // An element now looks like this:
    // 00000001 00000001(u16)
    // Now all bits are shifted to the right by (N=7), which gives us the following result:
    // 00000000 00000010(2)
    // Finally, the new result is accumulated to the value on the second vector and we get the following result:
    // 00000001 00000011(259)
    let shift_u16 = arm::vsraq_n_u16(vec_8x16, vec_8x16, 7);
    // We now turn the vector 8x-u16 into 4x-u32.
    // Here is an example of what happens to the elements:
    // 00000001 00000011(u16) + 
    // 00000001 00000011(u16) = 
    // 00000001 00000011 00000001 00000011(u32)
    let vec_4x32 = arm::vreinterpretq_u32_u16(shift_u16);

    // An element now looks like this:
    // 00000001 00000011 00000001 00000011(u32)
    // Now all bits are shifted to the right by (N=14), which gives us the following result:
    // 00000000 00000000 00000100 00001100(1036)
    // Finally, the new result is accumulated to the value on the second vector and we get the following result:
    // 00000001 00000011 00000101 00001111(16975119)
    let shift_u32 = arm::vsraq_n_u32(vec_4x32, vec_4x32, 14);
    // We now turn the vector 4x-u32 into 2x-u64.
    // Here is an example of what happens to the elements:
    // 00000001 00000011 00000101 00001111(u32) + 
    // 00000001 00000011 00000101 00001111(u32) = 
    // 00000001 00000011 00000101 00001111 00000001 00000011 00000101 00001111(u64)
    let vec_2x64 = arm::vreinterpretq_u64_u32(shift_u32);

    // An element now looks like this:
    // 00000001 00000011 00000101 00001111 00000001 00000011 00000101 00001111(u64)
    // Now all bits are shifted to the right by (N=28), which gives us the following result:
    // 00000000 00000000 00000000 00000000 00010000 00110000 01010000 11110000(271601904)
    // Finally, the new result is accumulated to the value on the second vector and we get the following result:
    // 00000001 00000011 00000101 00001111 00010001 00110011 01010101 11111111(72907581239285247)
    let shift_u64 = arm::vsraq_n_u64(vec_2x64, vec_2x64, 28);
    // Finally, we turn the vector 2x-u64 back into a 16x-u8.
    let vec_16x8 = arm::vreinterpretq_u8_u64(shift_u64);

    let (low, high): (u8, u8);
    #[cfg(target_endian = "little")]
    {
        // Our results ("low" and "high") are now in the first and ninth elements.
        (low, high) = (
            arm::vgetq_lane_u8(vec_16x8, 0),
            arm::vgetq_lane_u8(vec_16x8, 8),
        );
    }
    #[cfg(target_endian = "big")]
    {
        // To get the correct result with "big endian", we have to reverse the bits of all bytes.
        let reversed = arm::vrbitq_u8(vec_16x8);

        // Our results ("low" and "high") are now in the eighth and sixteenth elements.
        (low, high) = (
            arm::vgetq_lane_u8(reversed, 7),
            arm::vgetq_lane_u8(reversed, 15),
        );
    }
    
    // Our final result:
    ((high as u16) << 8) | (low as u16)
}

#[cfg(test)]
mod tests {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        target_feature = "avx2"
    ))]
    #[test]
    fn nnap_avx2() {
        const RESULTS: [(&[u8], usize); 2] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
        ];
        
        for (idx, result) in RESULTS.into_iter().enumerate() {
            let mut tmp = 0;
            let value = unsafe { super::nnap_avx2(result.0, &mut tmp) };
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        target_feature = "sse2"
    ))]
    #[test]
    fn nnap_sse2() {
        const RESULTS: [(&[u8], usize); 2] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
        ];
        
        for (idx, result) in RESULTS.into_iter().enumerate() {
            let mut tmp = 0;
            let value = unsafe { super::nnap_sse2(result.0, &mut tmp) };
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }

    #[cfg(all(
        any(target_arch = "aarch64", target_arch = "arm64ec"), 
        target_feature = "neon"
    ))]
    #[test]
    fn nnap_neon() {
        const RESULTS: [(&[u8], usize); 2] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
        ];
        
        for (idx, result) in RESULTS.into_iter().enumerate() {
            let mut tmp = 0;
            let value = unsafe { super::nnap_neon(result.0, &mut tmp) };
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }
}
