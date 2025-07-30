#[cfg(target_arch = "x86")]
use std::arch::x86;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as x86;
#[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
use core::arch::aarch64 as arm;

#[cfg(not(any(
    target_arch = "x86", target_arch = "x86_64",
    target_arch = "aarch64", target_arch = "arm64ec",
)))]
compile_error!("The current arch is not supported. Please disable the \"simd\" feature for \"utf-c\"!");

#[cfg(not(any(
    target_feature = "sse2",
    target_feature = "neon",
)))]
compile_error!("The required feature for SIMD are not available. Please disable the \"simd\" feature for \"utf-c\"!");

pub trait MaskValue: PartialEq + Default {
    /// Calls the `trailing_zeros` function of this type.
    fn trailing_zeros(self) -> u32;
}
impl_mask_value!(u16, i32, u64);

pub struct FindPositiveByteIndex<'a, 'b> {
    bytes: &'a [u8],
    index: &'b mut usize,
}

impl FindPositiveByteIndex<'_, '_> {
    pub const VEC_LEN: usize = 16;
    #[cfg(feature = "simd_extra")]
    pub const VEC_LEN_EXTRA: usize = 32;
    #[cfg(feature = "simd_ultra")]
    pub const VEC_LEN_ULTRA: usize = 64;

    fn r#loop<T, F>(&mut self, vec_len: usize, mask_cb: F) -> Option<usize> 
    where 
        T: MaskValue,
        F: Fn(&usize, *const u8) -> T,
    {
        let (len, ptr) = (self.bytes.len(), self.bytes.as_ptr());
        let end_idx = len - vec_len;

        while *self.index <= end_idx {
            let mask = mask_cb(self.index, ptr);

            if mask != T::default() {
                let result = *self.index + (mask.trailing_zeros() as usize);
                return Some(result);
            }

            *self.index += vec_len;
        }
        
        None
    }

    pub unsafe fn normal(&mut self) -> Option<usize> {
        if !feature_detected!(normal) {
            return None;
        }

        self.r#loop(Self::VEC_LEN, |&idx, ptr| {
            #[cfg(target_feature = "sse2")]
            unsafe {
                let simd_vec = x86::_mm_loadu_si128(ptr.add(idx) as *const x86::__m128i);
                x86::_mm_movemask_epi8(simd_vec)
            }

            #[cfg(target_feature = "neon")]
            unsafe {
                let simd_vec = arm::vld1q_u8(ptr.add(idx));
                neon_movemask_epu8(simd_vec)
            }
        })
    }

    #[cfg(feature = "simd_extra")]
    pub unsafe fn extra(&mut self) -> Option<usize> {
        #[cfg(not(target_feature = "avx2"))]
        compile_error!("A required SIMD instruction for your processor is missing. Please disable the \"simd_extra\" feature for \"utf-c\"!");

        if !feature_detected!(extra) {
            return None;
        }

        self.r#loop(Self::VEC_LEN_EXTRA, |&idx, ptr| {
            #[cfg(feature = "simd_extra")]
            unsafe {
                let simd_vec = x86::_mm256_loadu_si256(ptr.add(idx) as *const x86::__m256i);
                x86::_mm256_movemask_epi8(simd_vec)
            }
        })
    }

    #[cfg(feature = "simd_ultra")]
    pub unsafe fn ultra(&mut self) -> Option<usize> {
        #[cfg(not(target_feature = "avx512f"))]
        compile_error!("A required SIMD instruction for your processor is missing. Please disable the \"simd_ultra\" feature for \"utf-c\"!");

        if !feature_detected!(extra) {
            return None;
        }

        self.r#loop(Self::VEC_LEN_ULTRA, |&idx, ptr| {
            #[cfg(feature = "simd_ultra")]
            unsafe {
                let simd_vec = x86::_mm512_loadu_si512(ptr.add(idx) as *const x86::__m512i);
                x86::_mm512_movepi8_mask(simd_vec)
            }
        })
    }
}

impl<'a, 'b> From<(&'a [u8], &'b mut usize)> for FindPositiveByteIndex<'a, 'b> {
    #[inline]
    fn from(value: (&'a [u8], &'b mut usize)) -> Self {
        Self { bytes: value.0, index: value.1 }
    }
}

#[cfg(target_feature = "neon")]
/// An alternative to `_mm_movemask_epi8` (SSE2) for NEON.
/// 
/// [Click here for more details about `_mm_movemask_epi8`](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_movemask_epi8&ig_expand=4602)
unsafe fn neon_movemask_epu8(value: arm::uint8x16_t) -> u16 {
    /*
     * For more details and the "big endian" theory:
     * https://github.com/utf-c/neon_movemask_epu8/blob/main/README.md
     */
    
    let (low, high): (u8, u8);
    unsafe {
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
    } // unsafe
    ((high as u16) << 8) | (low as u16)
}

#[cfg(test)]
mod tests {
    const TEST_CASES: [(&[u8], usize); 2] = [
            (&[ 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], 5),
            (&[ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128 ], 31),
        ];
    
    #[test]
    fn fpbi_search() {
        for (idx, result) in TEST_CASES.into_iter().enumerate() {
            let mut skip = 0;
            let mut fpbi = super::FindPositiveByteIndex::from((result.0, &mut skip));
            let value = unsafe { fpbi.normal() };
            assert_eq!(value, Some(result.1), "failed at index {}", idx);

            #[cfg(feature = "simd_extra")]
            {
                let value = unsafe { fpbi.extra() };
                assert_eq!(value, Some(result.1), "failed at index {}", idx);
            }
        }
    }
}
