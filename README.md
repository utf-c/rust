# 0️⃣1️⃣0️⃣0️⃣0️⃣0️⃣1️⃣1️⃣
[![Crates.io Version](https://img.shields.io/crates/v/utf-c?style=flat-square)](https://crates.io/crates/utf-c)
[![MIT License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/utf-c/rust/blob/main/LICENSE)

UTF-C is a compression for short UTF-8 texts with non-ASCII characters (See the [comparisons](https://github.com/utf-c/rust?tab=readme-ov-file#comparisons) below).

> [!NOTE]
> The texts used here in various languages ​​were translated with [Google Translate](https://translate.google.com/).

## Examples
### `Ζω στην Ευρώπη`
In this example, we were able to remove 6 bytes.

```
Uncompressed(26): [    206, 150, 207, 137, 32, 207, 131, 207, 132, 206, 183, 206, 189, 32, 206, 149, 207, 133, 207, 129, 207, 142, 207, 128, 206, 183]
Compressed(20):   [26, 206, 150, 207, 137, 32,      131,      132, 206, 183,      189, 32,      149, 207, 133,      129,      142,      128, 206, 183]
```

### `私はヨーロッパに住んでいます`
In this example, we were able to remove 13 bytes.

```
Uncompressed(42): [    231, 167, 129, 227, 129, 175, 227, 131, 168, 227, 131, 188, 227, 131, 173, 227, 131, 131, 227, 131, 145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167, 227, 129, 132 227, 129, 190, 227, 129, 153]
Compressed(29):   [42, 231, 167, 129, 227, 129, 175, 227, 131, 168,           188,           173,           131,           145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167,           132,          190,           153]
```

## Comparisons
> [!IMPORTANT]
> Please create your own comparison and check if this compression is suitable for your project!

#### 📄 Cargo.toml
```toml
# ...

[dependencies]
utf-c = { path = "./utf-c/" }
# We will use `GzEncoder` and `GzDecoder`.
flate2 = "1.0.35"

[profile.release]
strip = true        # Automatically strip symbols from the binary
opt-level = 3       # Optimize for size
lto = true          # Enable link time optimization
codegen-units = 1   # Maximize size reduction optimizations
```

#### 🐧 Raspberry Pi OS Lite (64-bit Debian 12)<sup>Raspberry Pi 5</sup>
<ins>CPU:</ins> Broadcom BCM2712D0 quad-core Arm Cortex A76 processor @ 2.4GHz
<br>
<ins>SIMD:</ins> NEON
<br>
<ins>RAM:</ins> 1x 16GB LPDDR4X-4267

```
"👁👄👁" compression and decompression 50000x (12 bytes)
[flate2 | compression  ] finished after 926231 µs (31 bytes)
[flate2 | decompression] finished after 335222 µs
[utf-c  | compression  ] finished after 3325 µs (7 bytes)
[utf-c  | decompression] finished after 3546 µs
```
```
"טקסט זה נדחס עם UTF-C ו-GZIP ולאחר מכן הושווה. טקסט זה תורגם עם Google Translate ואנו מקווים שהוא תורגם כהלכה, אך אין ערובה לכך" compression and decompression 50000x (204 bytes)
[flate2 | compression  ] finished after 1507926 µs (160 bytes)
[flate2 | decompression] finished after 466058 µs
[utf-c  | compression  ] finished after 51067 µs (129 bytes)
[utf-c  | decompression] finished after 53226 µs
```

#### 🪟 Windows 11 Pro (24H2)
<ins>CPU:</ins> Intel Core i5-14400f (10c/16t)
<br>
<ins>SIMD:</ins> SSE2
<br>
<ins>RAM:</ins> 2x 8GB DDR4-3600

```
"👁👄👁" compression and decompression 50000x (12 bytes)
[flate2 | compression  ] finished after 363896 µs (31 bytes)
[flate2 | decompression] finished after 179868 µs
[utf-c  | compression  ] finished after 1755 µs (7 bytes)
[utf-c  | decompression] finished after 1887 µs
```
```
"טקסט זה נדחס עם UTF-C ו-GZIP ולאחר מכן הושווה. טקסט זה תורגם עם Google Translate ואנו מקווים שהוא תורגם כהלכה, אך אין ערובה לכך" compression and decompression 50000x (204 bytes)
[flate2 | compression  ] finished after 517129 µs (160 bytes)
[flate2 | decompression] finished after 235639 µs
[utf-c  | compression  ] finished after 15883 µs (129 bytes)
[utf-c  | decompression] finished after 18662 µs
```
