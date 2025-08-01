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
    pub const VEC_LEN_LEVEL1: usize = 16;
    #[cfg(feature = "simd_l2")]
    pub const VEC_LEN_LEVEL2: usize = 32;
    #[cfg(feature = "simd_l3")]
    pub const VEC_LEN_LEVEL3: usize = 64;

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

    pub unsafe fn level1(&mut self) -> Option<usize> {
        if !feature_detected!(level1) {
            return None;
        }

        self.r#loop(Self::VEC_LEN_LEVEL1, |&idx, ptr| {
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

    #[cfg(feature = "simd_l2")]
    pub unsafe fn level2(&mut self) -> Option<usize> {
        #[cfg(not(target_feature = "avx2"))]
        compile_error!("A required SIMD instruction for your processor is missing. Please disable the \"simd_l2\" feature for \"utf-c\"!");

        if !feature_detected!(level2) {
            return None;
        }

        self.r#loop(Self::VEC_LEN_LEVEL2, |&idx, ptr| {
            #[cfg(feature = "simd_l2")]
            unsafe {
                let simd_vec = x86::_mm256_loadu_si256(ptr.add(idx) as *const x86::__m256i);
                x86::_mm256_movemask_epi8(simd_vec)
            }
        })
    }

    #[cfg(feature = "simd_l3")]
    pub unsafe fn level3(&mut self) -> Option<usize> {
        #[cfg(not(target_feature = "avx512f"))]
        compile_error!("A required SIMD instruction for your processor is missing. Please disable the \"simd_l3\" feature for \"utf-c\"!");

        if !feature_detected!(level3) {
            return None;
        }

        self.r#loop(Self::VEC_LEN_LEVEL3, |&idx, ptr| {
            #[cfg(feature = "simd_l3")]
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
     * Details:
     * https://github.com/utf-c/neon_movemask_epu8/blob/v2/README.md
     */
    
    let (low, high): (u8, u8);
    
    unsafe {
        let msbs = arm::vshrq_n_u8(value, 7);

        const WEIGHTS_ARRAY: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];
        let weights = arm::vld1q_u8(WEIGHTS_ARRAY.as_ptr());
        let weighted = arm::vmulq_u8(msbs, weights);

        let sum16 = arm::vpaddlq_u8(weighted);
        let sum32 = arm::vpaddlq_u16(sum16);
        let sum64 = arm::vpaddlq_u32(sum32);

        #[cfg(target_endian = "little")]
        {
            (low, high) = (
                arm::vgetq_lane_u64(sum64, 0) as u8,
                arm::vgetq_lane_u64(sum64, 1) as u8
            );
        }
        #[cfg(target_endian = "big")]
        {
            let sum_bytes = arm::vreinterpretq_u8_u64(sum64);
            (low, high) = (
                arm::vgetq_lane_u8(sum_bytes, 7),
                arm::vgetq_lane_u8(sum_bytes, 15)
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
            {
                let mut skip = 0;
                let mut fpbi = super::FindPositiveByteIndex::from((result.0, &mut skip));
                let value = unsafe { fpbi.level1() };
                assert_eq!(value, Some(result.1), "failed at index {}", idx);
            }

            #[cfg(feature = "simd_l2")]
            {
                let mut skip = 0;
                let mut fpbi = super::FindPositiveByteIndex::from((result.0, &mut skip));
                let value = unsafe { fpbi.level2() };
                assert_eq!(value, Some(result.1), "failed at index {}", idx);
            }

            #[cfg(feature = "simd_l3")]
            {
                let mut skip = 0;
                let mut fpbi = super::FindPositiveByteIndex::from((result.0, &mut skip));
                let value = unsafe { fpbi.level3() };
                assert_eq!(value, Some(result.1), "failed at index {}", idx);
            }
        }
    }
}
