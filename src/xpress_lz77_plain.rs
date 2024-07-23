// Plain LZ77 Decompression Algorithm
// Reference: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/a8b7cb0a-92a6-4187-a23b-5e14273b96f8

use std::convert::TryInto;
use std::io::Error;

pub fn lz77_plain_decompress(in_buf: &[u8]) -> Result<Vec<u8>, Error> {
    let mut out_buf = Vec::new();
    let mut in_idx = 0;
    let mut out_idx = 0;
    let mut nibble_idx = 0;

    let mut flags = 0u32;
    let mut flag_count = 0;

    while in_idx < in_buf.len() {
        if flag_count == 0 {
            if in_buf.len() < in_idx + 4 {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Buffer is too short for flags",
                ));
            }
            flags = u32::from_le_bytes(in_buf[in_idx..in_idx + 4].try_into().unwrap());
            in_idx += 4;
            flag_count = 32;
        }

        flag_count -= 1;

        if (flags & (1 << flag_count)) == 0 {
            if in_idx >= in_buf.len() {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Input is too short",
                ));
            }
            out_buf.push(in_buf[in_idx]);
            in_idx += 1;
            out_idx += 1;
        } else {
            if in_idx >= in_buf.len() {
                return Ok(out_buf);
            }

            if in_buf.len() < in_idx + 2 {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Buffer is too short for length",
                ));
            }
            let length = u16::from_le_bytes(in_buf[in_idx..in_idx + 2].try_into().unwrap());
            in_idx += 2;

            let offset = (length / 8) as usize + 1;
            let mut length = (length % 8) as usize;

            if length == 7 {
                if nibble_idx == 0 {
                    if in_idx >= in_buf.len() {
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Buffer is too short for nibble",
                        ));
                    }
                    length = (in_buf[in_idx] % 16) as usize;
                    nibble_idx = in_idx;
                    in_idx += 1;
                } else {
                    length = (in_buf[nibble_idx] / 16) as usize;
                    nibble_idx = 0;
                }

                if length == 15 {
                    if in_idx >= in_buf.len() {
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Buffer is too short for extended length",
                        ));
                    }
                    length = in_buf[in_idx] as usize;
                    in_idx += 1;

                    if length == 255 {
                        if in_buf.len() < in_idx + 2 {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Buffer is too short for extended length",
                            ));
                        }
                        length = u16::from_le_bytes(in_buf[in_idx..in_idx + 2].try_into().unwrap())
                            as usize;
                        in_idx += 2;

                        if length == 0 {
                            if in_buf.len() < in_idx + 4 {
                                return Err(Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Buffer is too short for extended length",
                                ));
                            }
                            length =
                                u32::from_le_bytes(in_buf[in_idx..in_idx + 4].try_into().unwrap())
                                    as usize;
                            in_idx += 4;
                        }

                        if length < 15 + 7 {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "CorruptedData",
                            ));
                        }
                        length -= 15 + 7;
                    }
                    length += 15;
                }
                length += 7;
            }
            length += 3;

            for _ in 0..length {
                if offset > out_idx {
                    return Err(Error::new(std::io::ErrorKind::InvalidData, "CorruptedData"));
                }
                out_buf.push(out_buf[out_idx - offset]);
                out_idx += 1;
            }
        }
    }

    Ok(out_buf)
}
