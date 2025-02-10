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
    InvalidOrMissingPrefix(Vec<u8>),
}

/// Returns the compressed bytes or `CompressError`.
/// 
/// TIP: Use the `shrink_to_fit` function on the compressed bytes.
pub fn compress<T>(bytes: T) -> Result<Vec<u8>, CompressError> 
where 
    T: AsRef<[u8]>, 
{
    let mut value = bytes.as_ref();
    let value_len = value.len();
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

        if utf8_value.unicode() == utf8::Unicode::Unknown {
            // We found a non-ASCII character with an invalid or missing prefix.
            let err_result = value.iter().take(utf8::C_MAX_UTF8_BYTES).copied().collect::<Vec<u8>>();
            return Err(CompressError::InvalidOrMissingPrefix(err_result));
        }
        
        if utf8_value.unicode() == utf8::Unicode::Range00000_0007F {
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
    MissingBytes,
    MissingPrefix(Vec<u8>),
}

/// Returns the decompressed bytes or `DecompressError`.
pub fn decompress<T>(bytes: T) -> Result<Vec<u8>, DecompressError> 
where 
    T: AsRef<[u8]>, 
{
    let mut value = bytes.as_ref();
    let mut data_len = 0;
    for (idx, &len) in value.iter().enumerate() {
        data_len += len as usize;
        if len < u8::MAX {
            let new_idx = idx + 1;
            if value.len() < new_idx {
                return Err(DecompressError::MissingBytes);
            }
            value = &value[new_idx..];
            break;
        }
    }

    let mut result = Vec::<u8>::with_capacity(data_len);

    let mut last_utf8_prefix: &[u8] = &[];

    while !value.is_empty() {
        let utf8_value = utf8::Value::from(value);
        let (utf8_len, utf8_char): (usize, u8);
        
        if utf8_value.unicode() == utf8::Unicode::Unknown {
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
            if utf8_value.unicode() == utf8::Unicode::Range00000_0007F {
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
    const C_RESULTS: [(&str, &[u8], &[u8]); 7] = [
        (
            "",
            // UNCOMPRESSED(0)
            &[],
            // COMPRESSED(1)
            &[0],
        ),
        (
            "H",
            // UNCOMPRESSED(1)
            &[   72],
            // COMPRESSED(1)
            &[1, 72],
        ),
        (
            "HΉ",
            // UNCOMPRESSED(3)
            &[   72, 206, 137],
            // COMPRESSED(4)
            &[3, 72, 206, 137],
        ),
        (
            "HΉH",
            // UNCOMPRESSED(4)
            &[   72, 206, 137, 72],
            // COMPRESSED(5)
            &[4, 72, 206, 137, 72],
        ),
        (
            "HΉHΉ",
            // UNCOMPRESSED(6)
            &[   72, 206, 137, 72, 206, 137],
            // COMPRESSED(7)
            &[6, 72, 206, 137, 72,      137],
        ),
        (
            "ΉΉΉΉ",
            // UNCOMPRESSED(8)
            &[   206, 137, 206, 137, 206, 137, 206, 137],
            // COMPRESSED(6)
            &[8, 206, 137,      137,      137,      137],
        ),
        (
            "זו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הודעהזו הוהודעה",
            // UNCOMPRESSED(259)
            &[        215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148, 215, 150, 215, 149, 32, 215, 148, 215, 149, 215, 148, 215, 149, 215, 147, 215, 162, 215, 148],
            // COMPRESSED(141)
            &[255, 4, 215, 150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      147,      162,      148,      150,      149, 32,      148,      149,      148,      149,      147,      162,      148],
        ),
    ];

    #[test]
    fn compress() {
        for (text, uncompressed_bytes, compressed_bytes) in C_RESULTS {
            assert_eq!(text.as_bytes(), uncompressed_bytes);
            let result = super::compress(uncompressed_bytes).unwrap();
            assert_eq!(result.as_slice(), compressed_bytes);
        }
    }

    #[test]
    fn decompress() {
        for (text, uncompressed_bytes, compressed_bytes) in C_RESULTS {
            assert_eq!(text.as_bytes(), uncompressed_bytes);
            let result = super::decompress(compressed_bytes).unwrap();
            assert_eq!(result.as_slice(), uncompressed_bytes);
        }
    }
}
