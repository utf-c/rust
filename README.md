# 0️⃣1️⃣0️⃣0️⃣0️⃣0️⃣1️⃣1️⃣
UTF-C is a compression for short UTF-8 texts with non-ASCII characters.

> [!TIP]
> Use our `helper::only_ascii()` function (If possible together with the SIMD feature) to check if the bytes consist only of ASCII characters and skip compression.

### Example `Ζω στην Ευρώπη`
In this example, we were able to remove 5 bytes.

```
Uncompressed(26): [206, 150, 207, 137, 32, 207, 131, 207, 132, 206, 183, 206, 189, 32, 206, 149, 207, 133, 207, 129, 207, 142, 207, 128, 206, 183]
Compressed(21):   [206, 150, 207, 137, 32, 207, 131,      132, 206, 183,      189, 32, 206, 149, 207, 133,      129,      142,      128, 206, 183]
```

### Example `私はヨーロッパに住んでいます`
In this example, we were able to remove 14 bytes.

```
Uncompressed(42): [231, 167, 129, 227, 129, 175, 227, 131, 168, 227, 131, 188, 227, 131, 173, 227, 131, 131, 227, 131, 145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167, 227, 129, 132 227, 129, 190, 227, 129, 153]
Compressed(28):   [231, 167, 129, 227, 129, 175, 227, 131, 168,           188,           173,           131,           145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167,           132,          190,           153]
```

### Comparisons
> [!IMPORTANT]
> Please create your own comparison and check if this compression is suitable for your projects!

- The `GzEncoder` with `Compression::fast()` and `GzDecoder` was used for [flate2](https://crates.io/crates/flate2).
- All texts used were translated into different languages ​​by [Google Translate](https://translate.google.com/) with the content "I live in Europe" and "This text was translated with Google Translate for a comparison between UTF-C and GZIP!"

#### Cargo.toml
```toml
# ...

[dependencies]
# ...
flate2 = { version = "1.0.34", features = ["zlib-ng"], default-features = false }

[profile.release]
strip = true        # Automatically strip symbols from the binary
opt-level = 3       # Optimize for size
lto = true          # Enable link time optimization
codegen-units = 1   # Maximize size reduction optimizations
```

#### Results
```
"Ζω στην Ευρώπη" compression and decompression 50000x
[flate2 | compression  ] finished after 262907 µs
[flate2 | decompression] finished after 45830 µs
[utf-c  | compression  ] finished after 4515 µs
[utf-c  | decompression] finished after 6368 µs
========== flate2   (48) ==========
[31, 139, 8, 0, 0, 0, 0, 0, 4, 255, 59, 55, 237, 124, 167, 194, 249, 230, 243, 45, 231, 182, 159, 219, 171, 112, 110, 234, 249, 214, 243, 141, 231, 251, 206, 55, 156, 219, 14, 0, 107, 59, 158, 137, 26, 0, 0, 0]
========== utf-c    (21) ==========
[206, 150, 207, 137, 32, 207, 131, 132, 206, 183, 189, 32, 206, 149, 207, 133, 129, 142, 128, 206, 183]
========== original (26) ==========
[206, 150, 207, 137, 32, 207, 131, 207, 132, 206, 183, 206, 189, 32, 206, 149, 207, 133, 207, 129, 207, 142, 207, 128, 206, 183]
```
```
"私はヨーロッパに住んでいます" compression and decompression 50000x
[flate2 | compression  ] finished after 279333 µs
[flate2 | decompression] finished after 52958 µs
[utf-c  | compression  ] finished after 4650 µs
[utf-c  | decompression] finished after 7299 µs
========== flate2   (65) ==========
[31, 139, 8, 0, 0, 0, 0, 0, 4, 255, 123, 190, 188, 241, 113, 227, 250, 199, 205, 43, 30, 55, 239, 121, 220, 188, 246, 113, 115, 243, 227, 230, 137, 143, 27, 87, 63, 217, 219, 255, 184, 105, 242, 227, 198, 229, 143, 27, 91, 30, 55, 238, 123, 220, 56, 19, 0, 221, 68, 4, 112, 42, 0, 0, 0]
========== utf-c    (28) ==========
[231, 167, 129, 227, 129, 175, 227, 131, 168, 188, 173, 131, 145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167, 132, 190, 153]
========== original (42) ==========
[231, 167, 129, 227, 129, 175, 227, 131, 168, 227, 131, 188, 227, 131, 173, 227, 131, 131, 227, 131, 145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167, 227, 129, 132, 227, 129, 190, 227, 129, 153]
```
```
"ฉันอาศัยอยู่ในยุโรป" compression and decompression 50000x
[flate2 | compression  ] finished after 299444 µs
[flate2 | decompression] finished after 69296 µs
[utf-c  | compression  ] finished after 5549 µs
[utf-c  | decompression] finished after 7861 µs
========== flate2   (59) ==========
[31, 139, 8, 0, 0, 0, 0, 0, 4, 255, 123, 176, 163, 243, 193, 142, 153, 15, 118, 172, 125, 176, 99, 211, 131, 29, 43, 30, 236, 88, 4, 102, 47, 122, 176, 179, 25, 44, 190, 232, 193, 206, 166, 7, 59, 22, 63, 216, 49, 27, 0, 205, 177, 177, 184, 42, 0, 0, 0]
========== utf-c    (24) ==========
[224, 184, 137, 153, 173, 178, 168, 162, 173, 162, 224, 185, 131, 224, 184, 153, 162, 224, 185, 130, 224, 184, 163, 155]
========== original (42) ==========
[224, 184, 137, 224, 184, 153, 224, 184, 173, 224, 184, 178, 224, 184, 168, 224, 184, 162, 224, 184, 173, 224, 184, 162, 224, 185, 131, 224, 184, 153, 224, 184, 162, 224, 185, 130, 224, 184, 163, 224, 184, 155]
```
```
"Ζω στην Ευρώπη 私はヨーロッパに住んでいます ฉันอาศัยอยู่ในยุโรป" compression and decompression 50000x
[flate2 | compression  ] finished after 347787 µs
[flate2 | decompression] finished after 91696 µs
[utf-c  | compression  ] finished after 12195 µs
[utf-c  | decompression] finished after 15392 µs
========== flate2   (134) ==========
[31, 139, 8, 0, 0, 0, 0, 0, 4, 255, 59, 55, 237, 124, 167, 194, 249, 230, 243, 45, 231, 182, 159, 219, 171, 112, 110, 234, 249, 214, 243, 141, 231, 251, 206, 55, 156, 219, 174, 240, 124, 121, 227, 227, 198, 245, 143, 155, 87, 60, 110, 222, 243, 184, 121, 237, 227, 230, 230, 199, 205, 19, 31, 55, 174, 126, 178, 183, 255, 113, 211, 228, 199, 141, 203, 31, 55, 182, 60, 110, 220, 247, 184, 113, 166, 194, 131, 29, 157, 15, 118, 204, 124, 176, 99, 237, 131, 29, 155, 30, 236, 88, 241, 96, 199, 34, 48, 123, 209, 131, 157, 205, 96, 241, 69, 15, 118, 54, 61, 216, 177, 248, 193, 142, 217, 0, 117, 185, 227, 58, 112, 0, 0, 0]
========== utf-c    (75) ==========
[206, 150, 207, 137, 32, 207, 131, 132, 206, 183, 189, 32, 206, 149, 207, 133, 129, 142, 128, 206, 183, 32, 231, 167, 129, 227, 129, 175, 227, 131, 168, 188, 173, 131, 145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167, 132, 190, 153, 32, 224, 184, 137, 153, 173, 178, 168, 162, 173, 162, 224, 185, 131, 224, 184, 153, 162, 224, 185, 130, 224, 184, 163, 155]
========== original (112) ==========
[206, 150, 207, 137, 32, 207, 131, 207, 132, 206, 183, 206, 189, 32, 206, 149, 207, 133, 207, 129, 207, 142, 207, 128, 206, 183, 32, 231, 167, 129, 227, 129, 175, 227, 131, 168, 227, 131, 188, 227, 131, 173, 227, 131, 131, 227, 131, 145, 227, 129, 171, 228, 189, 143, 227, 130, 147, 227, 129, 167, 227, 129, 132, 227, 129, 190, 227, 129, 153, 32, 224, 184, 137, 224, 184, 153, 224, 184, 173, 224, 184, 178, 224, 184, 168, 224, 184, 162, 224, 184, 173, 224, 184, 162, 224, 185, 131, 224, 184, 153, 224, 184, 162, 224, 185, 130, 224, 184, 163, 224, 184, 155]
```
```
"טקסט זה תורגם באמצעות Google Translate לצורך השוואה בין UTF-C ו-GZIP!" compression and decompression 50000x
[flate2 | compression  ] finished after 338785 µs
[flate2 | decompression] finished after 98127 µs
[utf-c  | compression  ] finished after 14528 µs
[utf-c  | decompression] finished after 19677 µs
===== gzip     (124) =====
[31, 139, 8, 0, 0, 0, 0, 0, 4, 255, 187, 62, 227, 250, 242, 235, 11, 175, 207, 80, 184, 62, 237, 250, 20, 133, 235, 171, 174, 79, 189, 190, 226, 250, 164, 235, 115, 21, 174, 79, 188, 62, 225, 250, 188, 235, 203, 174, 47, 186, 62, 245, 250, 42, 5, 247, 252, 252, 244, 156, 84, 133, 144, 162, 196, 188, 226, 156, 196, 146, 84, 133, 235, 115, 174, 47, 3, 43, 158, 165, 112, 125, 202, 245, 149, 215, 167, 94, 159, 122, 125, 2, 200, 136, 137, 215, 103, 94, 159, 175, 16, 26, 226, 166, 235, 172, 112, 125, 170, 174, 123, 148, 103, 128, 34, 0, 169, 163, 170, 30, 102, 0, 0, 0]
===== utf-c    (77) =====
[215, 152, 167, 161, 152, 32, 215, 150, 148, 32, 215, 170, 149, 168, 146, 157, 32, 215, 145, 144, 158, 166, 162, 149, 170, 32, 71, 111, 111, 103, 108, 101, 32, 84, 114, 97, 110, 115, 108, 97, 116, 101, 32, 215, 156, 166, 149, 168, 154, 32, 215, 148, 169, 149, 149, 144, 148, 32, 215, 145, 153, 159, 32, 85, 84, 70, 45, 67, 32, 215, 149, 45, 71, 90, 73, 80, 33]
===== original (102) =====
[215, 152, 215, 167, 215, 161, 215, 152, 32, 215, 150, 215, 148, 32, 215, 170, 215, 149, 215, 168, 215, 146, 215, 157, 32, 215, 145, 215, 144, 215, 158, 215, 166, 215, 162, 215, 149, 215, 170, 32, 71, 111, 111, 103, 108, 101, 32, 84, 114, 97, 110, 115, 108, 97, 116, 101, 32, 215, 156, 215, 166, 215, 149, 215, 168, 215, 154, 32, 215, 148, 215, 169, 215, 149, 215, 149, 215, 144, 215, 148, 32, 215, 145, 215, 153, 215, 159, 32, 85, 84, 70, 45, 67, 32, 215, 149, 45, 71, 90, 73, 80, 33]
```