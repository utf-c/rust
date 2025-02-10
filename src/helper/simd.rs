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
unsafe fn fpbi_avx2(bytes: &[u8], skip: &mut usize) -> Option<usize> {
    const C_VEC_LEN: usize = 32;
    
    // We use this macro to check at runtime whether the CPU feature "avx2" is available.
    if is_x86_feature_detected!("avx2") {
        let (len, ptr) = (
            bytes.len(),
            bytes.as_ptr()
        );
        
        for idx in (0..len).step_by(C_VEC_LEN) {
            let simd_vec = x86::_mm256_loadu_si256(ptr.add(idx) as *const x86::__m256i);
            let mask = x86::_mm256_movemask_epi8(simd_vec);
            if mask != 0 {
                let result = idx + (mask.trailing_zeros() as usize);
                return Some(result);
            }
        }

        *skip = len % C_VEC_LEN;
    }
    None
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"), 
    target_feature = "sse2"
))]
unsafe fn fpbi_sse2(bytes: &[u8], skip: &mut usize) -> Option<usize> {
    const C_VEC_LEN: usize = 16;

    // We use this macro to check at runtime whether the CPU feature "sse2" is available.
    if is_x86_feature_detected!("sse2") {
        let (len, ptr) = (
            bytes.len(),
            bytes.as_ptr()
        );

        for idx in (*skip..len).step_by(C_VEC_LEN) {
            let simd_vec = x86::_mm_loadu_si128(ptr.add(idx) as *const x86::__m128i);
            let mask = x86::_mm_movemask_epi8(simd_vec);
            if mask != 0 {
                let result = idx + (mask.trailing_zeros() as usize);
                return Some(result);
            }
        }

        *skip = len % C_VEC_LEN;
    }
    None
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm64ec"), 
    target_feature = "neon"
))]
#[inline(always)]
unsafe fn fpbi_neon(bytes: &[u8], skip: &mut usize) -> Option<usize> {
    const C_VEC_LEN: usize = 16;

    // We use this macro to check at runtime whether the CPU feature "neon" is available.
    if std::arch::is_aarch64_feature_detected!("neon") {
        let (len, ptr) = (
            bytes.len(),
            bytes.as_ptr()
        );
        
        for idx in (0..len).step_by(C_VEC_LEN) {
            let simd_vec = arm::vld1q_u8(ptr.add(idx));
            let mask = neon_movemask_epu8(simd_vec);
            if mask != 0 {
                let result = idx + (mask.trailing_zeros() as usize);
                return Some(result);
            }
        }

        *skip = len % C_VEC_LEN;
    }
    None
}

pub unsafe fn find_pos_byte_idx(bytes: &[u8]) -> Option<usize> {
    let len = bytes.len();
    // The number of bytes to skip.
    // As an example: If AVX2 has already checked 32 bytes, SSE2 should skip these 32.
    let mut skip = 0;

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        target_feature = "avx2"
    ))]
    if len >= 32 {
        let result = fpbi_avx2(bytes, &mut skip);
        if result.is_some() {
            return result;
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        target_feature = "sse2"
    ))]
    if (len - skip) >= 16 {
        let result = fpbi_sse2(bytes, &mut skip);
        if result.is_some() {
            return result;
        }
    }

    #[cfg(all(
        any(target_arch = "aarch64", target_arch = "arm64ec"), 
        target_feature = "neon"
    ))]
    if len >= 16 {
        let result = fpbi_neon(bytes, &mut skip);
        if result.is_some() {
            return result;
        }
    }
    
    // Now check the last (less than 16 or 32) bytes, with a normal loop.
    (skip..len).find(|&i| super::test_sign_bit(bytes[i]))
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm64ec"), 
    target_feature = "neon"
))]
/// An alternative to `_mm_movemask_epi8` (SSE2) for NEON.
/// 
/// [Click here for more details about `_mm_movemask_epi8`](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_movemask_epi8&ig_expand=4602)
unsafe fn neon_movemask_epu8(value: arm::uint8x16_t) -> u16 {
    /*
     * For more details and the "big endian" theory:
     * https://github.com/utf-c/neon_movemask_epu8/blob/main/README.md
     */
    
    // We shift the bits of all elements (N=7) to the right.
    let shift_u8 = arm::vshrq_n_u8(value, 7);
    // We now turn the vector 16x-u8 into a 8x-u16.
    let vec_8x16 = arm::vreinterpretq_u16_u8(shift_u8);

    // We now pass the same vector twice to shift the bits of all elements
    // in one of these vectors (N=7) to the right and accumulate them with the other.
    let shift_u16 = arm::vsraq_n_u16(vec_8x16, vec_8x16, 7);
    // We now turn the vector 8x-u16 into a 4x-u32.
    let vec_4x32 = arm::vreinterpretq_u32_u16(shift_u16);

    // We now pass the same vector twice to shift the bits of all elements
    // in one of these vectors (N=14) to the right and accumulate them with the other.
    let shift_u32 = arm::vsraq_n_u32(vec_4x32, vec_4x32, 14);
    // We now turn the vector 4x-u32 into a 2x-u64.
    let vec_2x64 = arm::vreinterpretq_u64_u32(shift_u32);

    // We now pass the same vector twice to shift the bits of all elements
    // in one of these vectors (N=28) to the right and accumulate them with the other.
    let shift_u64 = arm::vsraq_n_u64(vec_2x64, vec_2x64, 28);
    // Finally, we turn the vector 2x-u64 back into a 16x-u8.
    let vec_16x8 = arm::vreinterpretq_u8_u64(shift_u64);

    // Now we extract two elements (our "low" and "high") to get our result.
    // NOTE: To understand why there are differences here, look at the theory.
    let (low, high): (u8, u8);
    #[cfg(target_endian = "little")]
    {
        (low, high) = (
            arm::vgetq_lane_u8(vec_16x8, 0),
            arm::vgetq_lane_u8(vec_16x8, 8),
        );
    }
    #[cfg(target_endian = "big")]
    {
        // To get the correct result with "big endian", we have to reverse the bits of all elements.
        let reversed = arm::vrbitq_u8(vec_16x8);
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
    fn fpbi_avx2() {
        const C_RESULTS: [(&[u8], usize); 2] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
        ];
        
        for (idx, result) in C_RESULTS.into_iter().enumerate() {
            let mut tmp = 0;
            let value = unsafe { super::fpbi_avx2(result.0, &mut tmp) };
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"), 
        target_feature = "sse2"
    ))]
    #[test]
    fn fpbi_sse2() {
        const C_RESULTS: [(&[u8], usize); 2] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
        ];
        
        for (idx, result) in C_RESULTS.into_iter().enumerate() {
            let mut tmp = 0;
            let value = unsafe { super::fpbi_sse2(result.0, &mut tmp) };
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }

    #[cfg(all(
        any(target_arch = "aarch64", target_arch = "arm64ec"), 
        target_feature = "neon"
    ))]
    #[test]
    fn fpbi_neon() {
        const C_RESULTS: [(&[u8], usize); 2] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
        ];
        
        for (idx, result) in C_RESULTS.into_iter().enumerate() {
            let mut tmp = 0;
            let value = unsafe { super::fpbi_neon(result.0, &mut tmp) };
            assert_eq!(value, Some(result.1), "failed at index {}", idx);
        }
    }
}
