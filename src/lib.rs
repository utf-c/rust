pub mod helper;

mod char_set;
use char_set::CharSetType;

/// Returns `false` if all characters are ASCII, otherwise `true`.
#[inline]
fn _is_ascii_then(value: &mut &[u8], result: &mut Vec<u8>) -> bool {
    // Now check up to which position it is no longer an ASCII character
    if let Some(next_pos) = helper::next_non_ascii_pos(value) {
        // Position found, now add all bytes up to this position and continue the loop
        result.extend_from_slice(&value[..next_pos]);
        *value = &value[next_pos..];
        return true;
    }

    // No non-ASCII character found, so all remaining characters are ASCII characters
    result.extend_from_slice(value);
    false
}

pub fn compress(mut value: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
    let mut result = Vec::<u8>::with_capacity(value.len());

    let mut last_set: &[u8] = &[];

    while !value.is_empty() {
        let cst = CharSetType::from(value);

        if cst == CharSetType::Unknown {
            // We found a non-ASCII character with an invalid or missing set
            let err_result = value.iter().take(char_set::C_MAX_LEN).copied().collect::<Vec<u8>>();
            return Err(err_result);
        }
        
        if cst == CharSetType::Unicode00000_0007f {
            match _is_ascii_then(&mut value, &mut result) {
                false => break,
                true => continue
            };
        }
        
        let (cst_set, cst_len) = (cst.set(), cst.len());
        if last_set != cst_set {
            last_set = cst_set;
            result.extend_from_slice(cst_set);
        }

        result.push(cst.value());
        value = &value[cst_len..];
    }
    
    Ok(result)
}

pub fn decompress(mut value: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
    let mut result = Vec::<u8>::with_capacity(value.len());

    let mut last_set: &[u8] = &[];

    while !value.is_empty() {
        let cst = CharSetType::from(value);
        let (cst_len, cst_value): (usize, u8);
        if cst == CharSetType::Unknown {
            // We have found a CharSetType::Unknown,
            // which means we have a character with the same last set.

            if last_set.is_empty() {
                // Should only happen if there was no set for the first non-ASCII character,
                // as in this example: &[ 72, 101, 108, 108, 111, 32, 149 ]
                //                                                    ^-Non-ASCII character without a set
                let err_result = value.iter().take(char_set::C_MAX_LEN).copied().collect::<Vec<u8>>();
                return Err(err_result);
            }

            cst_len = 1;
            cst_value = value[0];
        } else {
            if cst == CharSetType::Unicode00000_0007f {
                match _is_ascii_then(&mut value, &mut result) {
                    false => break,
                    true => continue
                };
            }

            last_set = cst.set();
            cst_len = cst.len();
            cst_value = cst.value();
        }

        result.extend_from_slice(last_set);
        result.push(cst_value);

        value = &value[cst_len..];
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    const RESULTS: [(&str, &[u8], &[u8]); 6] = [
        (
            "Hello",
            // UNCOMPRESSED(5)
            &[72, 101, 108, 108, 111],
            // COMPRESSED(5)
            &[72, 101, 108, 108, 111]
        ),
        (
            "Hello ÑÍÇ½",
            // UNCOMPRESSED(14)
            &[72, 101, 108, 108, 111, 32, 195, 145, 195, 141, 195, 135, 194, 189],
            // COMPRESSED(12)
            &[72, 101, 108, 108, 111, 32, 195, 145,      141,      135, 194, 189]
        ),
        (
            "Hello ÑÍÇ½ Hello",
            // UNCOMPRESSED(20)
            &[72, 101, 108, 108, 111, 32, 195, 145, 195, 141, 195, 135, 194, 189, 32, 72, 101, 108, 108, 111],
            // COMPRESSED(18)
            &[72, 101, 108, 108, 111, 32, 195, 145,      141,      135, 194, 189, 32, 72, 101, 108, 108, 111]
        ),
        (
            "Hello ÑÍÇ½ Hello ÑÍÇ½",
            // UNCOMPRESSED(29)
            &[72, 101, 108, 108, 111, 32, 195, 145, 195, 141, 195, 135, 194, 189, 32, 72, 101, 108, 108, 111, 32, 195, 145, 195, 141, 195, 135, 194, 189],
            // COMPRESSED(25)
            &[72, 101, 108, 108, 111, 32, 195, 145,      141,      135, 194, 189, 32, 72, 101, 108, 108, 111, 32, 195, 145,      141,      135, 194, 189]
        ),
        (
            "Ήταν μια πολύ καλή μέρα.",
            // UNCOMPRESSED(43)
            &[206, 137, 207, 132, 206, 177, 206, 189, 32, 206, 188, 206, 185, 206, 177, 32, 207, 128, 206, 191, 206, 187, 207, 141, 32, 206, 186, 206, 177, 206, 187, 206, 174, 32, 206, 188, 206, 173, 207, 129, 206, 177, 46],
            // COMPRESSED(33)
            &[206, 137, 207, 132, 206, 177,      189, 32,      188,      185,      177, 32, 207, 128, 206, 191,      187, 207, 141, 32, 206, 186,      177,      187,      174, 32,      188,      173, 207, 129, 206, 177, 46],
        ),
        (
            "The Symbols ﹩﹫﹪ are not $@%",
            // UNCOMPRESSED(33)
            &[84, 104, 101, 32, 83, 121, 109, 98, 111, 108, 115, 32, 239, 185, 169, 239, 185, 171, 239, 185, 170, 32, 97, 114, 101, 32, 110, 111, 116, 32, 36, 64, 37],
            // COMPRESSED(29)
            &[84, 104, 101, 32, 83, 121, 109, 98, 111, 108, 115, 32, 239, 185, 169,           171,           170, 32, 97, 114, 101, 32, 110, 111, 116, 32, 36, 64, 37]
        )
    ];

    #[test]
    fn compress() {
        for (text, uncompressed_bytes, compressed_bytes) in RESULTS {
            assert_eq!(text.as_bytes(), uncompressed_bytes);
            let result = super::compress(uncompressed_bytes).unwrap();
            assert_eq!(result.as_slice(), compressed_bytes);
        }
    }

    #[test]
    fn decompress() {
        for (text, uncompressed_bytes, compressed_bytes) in RESULTS {
            assert_eq!(text.as_bytes(), uncompressed_bytes);
            let result = super::decompress(compressed_bytes).unwrap();
            assert_eq!(result.as_slice(), uncompressed_bytes);
        }
    }
}
