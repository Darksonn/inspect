use std::io::{BufRead, Read, Error, ErrorKind};
use std::fmt::{self, Write};

pub struct Base64<I> {
    state: State,
    decoded: Decoded,
    inner: I,
    buffer: Vec<u8>,
    length: usize,
    offset: usize,
}

impl<I: BufRead> Base64<I> {
    pub fn new(inner: I) -> Self {
        Self {
            state: State::new(),
            decoded: Decoded::empty(),
            buffer: Vec::new(),
            length: 0,
            offset: 0,
            inner,
        }
    }
}
impl<I: BufRead> Read for Base64<I> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut written = 0;
        let buf_len = buf.len();
        for ptr in buf {
            while self.decoded.len == 0 {
                let digit = if self.length > self.offset {
                    let off = self.offset;
                    self.offset = off + 1;
                    self.buffer[off]
                } else {
                    let to_read = buf_len - written;
                    if self.buffer.len() < to_read {
                        self.buffer.resize(to_read, 0);
                    }
                    self.length = self.inner.read(&mut self.buffer[..to_read])?;
                    if self.length == 0 {
                        self.offset = 0;
                        return Ok(written);
                    }
                    self.offset = 1;
                    self.buffer[0]
                };

                let digit = match Base64Digit::from(digit) {
                    Some(digit) => digit,
                    None => continue,
                };
                self.decoded = self.state.add_state(digit)?;
            }
            *ptr = self.decoded.get();
            written += 1;
        }
        Ok(written)
    }
}

/// A value from 0 to 63 or 255 for padding.
#[derive(Copy,Clone)]
struct Base64Digit {
    value: u8,
}
impl Base64Digit {
    fn from(byte: u8) -> Option<Self> {
        let i = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'-' => 62,
            b'_' => 63,
            b'=' => 255,
            _ => return None,
        };
        Some(Base64Digit {
            value: i,
        })
    }
    fn is_padding(self) -> bool {
        self.value >= 64
    }
}

struct Decoded {
    vals: [u8; 3],
    len: u8,
}
impl Decoded {
    fn empty() -> Self {
        Self {
            vals: [0, 0, 0],
            len: 0,
        }
    }
    fn get(&mut self) -> u8 {
        let res = self.vals[0];
        self.vals[0] = self.vals[1];
        self.vals[1] = self.vals[2];
        self.len -= 1;
        res
    }
}
struct State {
    seen: [Base64Digit; 3],
    len: u8,
}
impl State {
    fn new() -> State {
        State {
            seen: [
                Base64Digit { value: 0 },
                Base64Digit { value: 0 },
                Base64Digit { value: 0 },
            ],
            len: 0,
        }
    }
    fn add_state(&mut self, digit: Base64Digit) -> Result<Decoded, Base64Error> {
        match self.len {
            0 => {
                if digit.is_padding() {
                    return Err(Base64Error);
                }
                self.len = 1;
                self.seen[0] = digit;
                Ok(Decoded::empty())
            }
            1 => {
                if digit.is_padding() {
                    return Err(Base64Error);
                }
                self.len = 2;
                self.seen[1] = digit;
                Ok(Decoded::empty())
            }
            2 => {
                self.len = 3;
                self.seen[2] = digit;
                Ok(Decoded::empty())
            }
            3 => {
                self.len = 0;
                if self.seen[2].is_padding() {
                    if !digit.is_padding() {
                        return Err(Base64Error);
                    }
                    let byte = self.seen[0].value << 2
                        | ((self.seen[1].value >> 4) & 0b11);
                    Ok(Decoded {
                        vals: [byte, 0, 0],
                        len: 1,
                    })
                } else if digit.is_padding() {
                    let byte1 = self.seen[0].value << 2
                        | ((self.seen[1].value >> 4) & 0b11);
                    let byte2 = (self.seen[1].value & 0b1111) << 4
                        | ((self.seen[2].value >> 2) & 0b1111);
                    Ok(Decoded {
                        vals: [byte1, byte2, 0],
                        len: 2,
                    })
                } else {
                    let byte1 = self.seen[0].value << 2
                        | ((self.seen[1].value >> 4) & 0b11);
                    let byte2 = (self.seen[1].value & 0b1111) << 4
                        | ((self.seen[2].value >> 2) & 0b1111);
                    let byte3 = (self.seen[2].value & 0b11) << 6
                        | digit.value;
                    Ok(Decoded {
                        vals: [byte1, byte2, byte3],
                        len: 3,
                    })
                }
            }
            _ => panic!("Invalid state"),
        }
    }
}

#[derive(Debug)]
struct Base64Error;
impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid base64 padding.")
    }
}
impl std::error::Error for Base64Error { }
impl From<Base64Error> for Error {
    fn from(e: Base64Error) -> Error {
        Error::new(ErrorKind::InvalidData, e)
    }
}
