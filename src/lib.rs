/* private modules */
mod utf8;

/* public modules */
pub mod helper;

/// Returns `true` if all characters were ASCII and were successfully processed, otherwise `false`.
#[inline(always)]
fn handle_ascii(value: &mut &[u8], result: &mut Vec<u8>) -> bool {
    // Check if only one character is left or if the second is a non-ASCII character.
    if value.len() == 1 || helper::test_sign_bit(value[1]) {
        result.push(value[0]);
        *value = &value[1..];
        return false;
    }

    // Now check up to which index it is no longer an ASCII character.
    if let Some(next_idx) = helper::find_pos_byte_idx(value) {
        // Index found, now add all bytes up to this index.
        result.extend_from_slice(&value[..next_idx]);
        *value = &value[next_idx..];
        return false;
    }

    // No non-ASCII character found, so all remaining characters are ASCII characters.
    result.extend_from_slice(value);
    true
}

#[derive(Debug)]
pub enum CompressError {
    InvalidLength,
    InvalidOrMissingPrefix(Vec<u8>),
}

/// Returns the compressed bytes or `CompressError`.
/// 
/// __TIP:__ Use the [`shrink_to_fit`](https://doc.rust-lang.org/beta/alloc/vec/struct.Vec.html#method.shrink_to_fit) function on the compressed bytes.
/// 
/// # Example
/// ```
/// const TEXT: &str = "ÄÖÜ";
/// const BYTES: [u8; 5] = [6, 195, 132, 150, 156];
/// //                      |  |    |-Ä  |-Ö  |-Ü
/// //                      |  |-Prefix
/// //                      |-Length
/// let mut result = utf_c::compress(TEXT).unwrap();
/// assert_eq!(result, BYTES);
/// ```
pub fn compress<T>(bytes: T) -> Result<Vec<u8>, CompressError> 
where 
    T: AsRef<[u8]>, 
{
    let mut value = bytes.as_ref();
    let value_len = value.len();
    if value_len == 0 {
        return Err(CompressError::InvalidLength);
    }

    let data_len_count = value_len / 255;
    let data_len_remainder = value_len % 255;

    let mut result = Vec::<u8>::with_capacity(data_len_count + 1 + value_len);
    if data_len_count > 0 {
        // We can use the unsafe functions, because we are using a larger
        // capacity and this is our first data for this vector.
        unsafe {
            result.set_len(data_len_count);
            result.as_mut_ptr().write_bytes(255, data_len_count);
        }
    }
    result.push(data_len_remainder as u8);

    let mut last_utf8_prefix: &[u8] = &[];

    while !value.is_empty() {
        let utf8_value = utf8::Value::from(value);
        let utf8_unicode = utf8_value.unicode();

        if utf8_unicode == utf8::Unicode::Unknown {
            // We found a non-ASCII character with an invalid or missing prefix.
            let err_result = value.iter().take(utf8::C_MAX_UTF8_BYTES).copied().collect::<Vec<u8>>();
            return Err(CompressError::InvalidOrMissingPrefix(err_result));
        }
        
        if utf8_unicode == utf8::Unicode::Range00000_0007F {
            if handle_ascii(&mut value, &mut result) {
                break;
            }
            continue;
        }
        
        let (utf8_prefix, utf8_len) = (utf8_value.prefix(), utf8_value.len());
        if last_utf8_prefix != utf8_prefix {
            last_utf8_prefix = utf8_prefix;
            result.extend_from_slice(utf8_prefix);
        }

        result.push(utf8_value.char());
        // We can use the unsafe function `get_unchecked`,
        // because we know that `value` is at least `utf8_len` long.
        unsafe {
            value = value.get_unchecked(utf8_len..);
        }
    }
    
    Ok(result)
}

#[derive(Debug)]
pub enum DecompressError {
    InvalidLength,
    MissingBytes,
    MissingPrefix(Vec<u8>),
}

/// Returns the decompressed bytes or `DecompressError`.
/// 
/// # Example
/// ```
/// const BYTES: [u8; 5] = [6, 195, 132, 150, 156];
/// //                      |  |    |-Ä  |-Ö  |-Ü
/// //                      |  |-Prefix
/// //                      |-Length
/// let result = utf_c::decompress(BYTES).unwrap();
/// assert_eq!(result, "ÄÖÜ".as_bytes());
/// ```
pub fn decompress<T>(bytes: T) -> Result<Vec<u8>, DecompressError> 
where 
    T: AsRef<[u8]>, 
{
    let mut value = bytes.as_ref();
    let value_len = value.len();
    if value_len < 2 {
        return Err(DecompressError::InvalidLength);
    }

    let data_len = unsafe {
        let mut idx = 0;
        loop {
            if value_len == idx {
                return Err(DecompressError::MissingBytes);
            }
            if *value.get_unchecked(idx) < 255 {
                break;
            }
            idx += 1;
        }
        let remainder = *value.get_unchecked(idx);
        value = value.get_unchecked((idx + 1)..);
        (idx * 255) + (remainder as usize)
    };

    let mut result = Vec::<u8>::with_capacity(data_len);

    let mut last_utf8_prefix: &[u8] = &[];

    while !value.is_empty() {
        let utf8_value = utf8::Value::from(value);
        let utf8_unicode = utf8_value.unicode();
        let (utf8_len, utf8_char): (usize, u8);
        
        if utf8_unicode == utf8::Unicode::Unknown {
            // We have found a utf8::Unicode::Unknown,
            // which means we have a character with the same last prefix.

            if last_utf8_prefix.is_empty() {
                // Should only happen if there was no set for the first non-ASCII character,
                // as in this example: &[ 72, 101, 108, 108, 111, 32, 149 ]
                //                                                    ^ Non-ASCII character without a prefix.
                let err_result = value.iter().take(utf8::C_MAX_UTF8_BYTES).copied().collect::<Vec<u8>>();
                return Err(DecompressError::MissingPrefix(err_result));
            }

            utf8_len = 1;
            utf8_char = value[0];
        } else {
            if utf8_unicode == utf8::Unicode::Range00000_0007F {
                if handle_ascii(&mut value, &mut result) {
                    break;
                }
                continue;
            }

            utf8_len = utf8_value.len();
            utf8_char = utf8_value.char();
            last_utf8_prefix = utf8_value.prefix();
        }

        result.extend_from_slice(last_utf8_prefix);
        result.push(utf8_char);
        // We can use the unsafe function `get_unchecked`,
        // because we know that `value` is at least `utf8_len` long.
        unsafe {
            value = value.get_unchecked(utf8_len..);
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn compress_and_decompress() {
        let test_cases: [(&str, &[u8]); 6] = [
            ("H", &[1, 72]),
            ("Hello world", &[11, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]),
            ("שלום עולם", &[17, 215, 169, 156, 149, 157, 32, 162, 149, 156, 157]),
            ("Hello עוֹלָם", &[18, 72, 101, 108, 108, 111, 32, 215, 162, 149, 214, 185, 215, 156, 214, 184, 215, 157]),
            (&"a".repeat(512), {
                let prefix: Vec<u8> = vec![255, 255, 2 ];
                let result: Vec<u8> = [97].repeat(512);
                &[prefix, result].concat()
            }),
            (&"α".repeat(512), {
                let prefix: Vec<u8> = vec![255, 255, 255, 255, 4, 206 ];
                let result: Vec<u8> = [177].repeat(512);
                &[prefix, result].concat()
            }),
        ];

        for (text, pre_compressed_bytes) in test_cases {
            // Compression
            let compressed_result = super::compress(text);
            assert!(compressed_result.is_ok(), "Compression failed for: {}", text);
            let compressed_bytes = compressed_result.unwrap();
            assert!(compressed_bytes == pre_compressed_bytes, "Compressed bytes does not match for: {:?}", compressed_bytes);

            // Decompression
            let decompressed_result = super::decompress(&compressed_bytes);
            assert!(decompressed_result.is_ok(), "Decompression failed for: {}", text);
            let decompressed_bytes = decompressed_result.unwrap();
            assert!(decompressed_bytes == text.as_bytes(), "Decompressed bytes does not match for: {}", text);
        }
    }

    #[test]
    fn compress_invalid_input() {
        let test_cases: [&[u8]; 2] = [
            &[],           // InvalidLength
            &[0b10000000], // InvalidOrMissingPrefix([128])
        ];

        for invalid_data in test_cases {
            let result = super::compress(invalid_data);
            assert!(result.is_err(), "Compression should have failed for {:?}", invalid_data);
        }
    }

    #[test]
    fn decompress_invalid_input() {
        let test_cases: [&[u8]; 3] = [
            &[],         // InvalidLength
            &[255, 255], // MissingBytes
            &[1, 149],   // MissingPrefix([149])
        ];

        for invalid_data in test_cases {
            let result = super::decompress(invalid_data);
            assert!(result.is_err(), "Decompression should have failed for {:?}", invalid_data);
        }
    }
}