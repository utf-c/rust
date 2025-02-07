pub mod helper;

mod char_set;
use char_set::CharSetType;

/// Returns `false` if all characters are ASCII, otherwise `true`.
#[inline(always)]
fn _is_ascii_then(value: &mut &[u8], result: &mut Vec<u8>) -> bool {
    // Check if only one character is left or if the second is a non-ASCII character.
    if value.len() == 1 || (value[1] >> 7) != 0 {
        result.push(value[0]);
        *value = &value[1..];
        return true;
    }

    // Now check up to which index it is no longer an ASCII character.
    if let Some(next_pos) = helper::next_non_ascii_idx(*value) {
        // Index found, now add all bytes up to this index and continue the loop.
        result.extend_from_slice(&value[..next_pos]);
        *value = &value[next_pos..];
        return true;
    }

    // No non-ASCII character found, so all remaining characters are ASCII characters.
    result.extend_from_slice(value);
    false
}

pub fn compress<T>(buf: T) -> Result<Vec<u8>, Vec<u8>> 
where 
    T: AsRef<[u8]>, 
{
    let mut value = buf.as_ref();
    let value_len = value.len();
    let value_len_count = value_len / (u8::MAX as usize);

    let mut result = Vec::<u8>::with_capacity(1 + value_len_count + value_len);
    if value_len_count > 0 {
        result.extend(vec![u8::MAX; value_len_count]);
    }
    let value_len_remainder = value_len - ((u8::MAX as usize) * value_len_count);
    result.push(value_len_remainder as u8);

    let mut last_set: &[u8] = &[];

    while !value.is_empty() {
        let cst = CharSetType::from(value);

        if cst == CharSetType::Unknown {
            // We found a non-ASCII character with an invalid or missing set.
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

pub fn decompress<T>(buf: T) -> Result<Vec<u8>, Vec<u8>> 
where 
    T: AsRef<[u8]>, 
{
    let mut value = buf.as_ref();
    let mut value_len = 0;
    for i in 0..usize::MAX {
        let len = value[i];

        value_len += len as usize;

        if len < u8::MAX {
            value = &value[(i + 1)..];
            break;
        }
    }

    let mut result = Vec::<u8>::with_capacity(value_len);

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
    const RESULTS: [(&str, &[u8], &[u8]); 7] = [
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
