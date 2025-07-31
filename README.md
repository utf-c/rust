# 0️⃣1️⃣0️⃣0️⃣0️⃣0️⃣1️⃣1️⃣
[![Crates.io Version](https://img.shields.io/crates/v/utf-c?style=flat-square)](https://crates.io/crates/utf-c)
[![MIT License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/utf-c/rust/blob/main/LICENSE)

UTF-C is a compression for short UTF-8 texts with non-ASCII characters (See the [comparisons](https://github.com/utf-c/rust?tab=readme-ov-file#comparisons) below).

> [!NOTE]
> Support for SSE2, AVX2, AVX512 and NEON.
>
> Check out the features in the [`Cargo.toml`](https://github.com/utf-c/rust/blob/main/Cargo.toml)!

## Comparison
> [!IMPORTANT]
> Please create your own comparison and check if this compression is suitable for your project!

#### 🪟 Windows 11 Pro (24H2)
<ins>CPU:</ins> Intel Core i5-14400f (10c/16t)
<br>
<ins>SIMD:</ins> SSE2
<br>
<ins>RAM:</ins> 2x 8GB DDR4-3600

```
TEXT: טקסט זה נדחס עם UTF-C ו-GZIP ולאחר מכן הושווה. טקסט זה תורגם עם Google Translate ואנו מקווים שהוא תורגם כהלכה, אך אין ערובה לכך
LENGTH: 204 (Original) | 129 (UTF-C) | 160 (FLATE2)

=============================================
                    UTF-C                    
=============================================
compression:   [340.83 ns 341.22 ns 341.64 ns] [569.45 MiB/s 570.15 MiB/s 570.82 MiB/s]
decompression: [368.41 ns 369.50 ns 370.86 ns] [524.58 MiB/s 526.53 MiB/s 528.07 MiB/s]

=============================================
                   FLATE2                    
=============================================
compression:   [11.582 µs 11.613 µs 11.650 µs] [16.700 MiB/s 16.752 MiB/s 16.798 MiB/s]
decompression: [5.0996 µs 5.1130 µs 5.1298 µs] [37.926 MiB/s 38.050 MiB/s 38.150 MiB/s]
```