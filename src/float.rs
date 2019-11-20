use std::io::{Read, Result, StdoutLock, Write};
use std::fmt::{Write as FmtWrite};

pub enum Kind {
    F32, F64
}
impl Kind {
    fn get_size(&self) -> usize {
        match self {
            F32 => 4,
            F64 => 8,
        }
    }
    fn parse(&self, bytes: [u8; 8]) -> f64 {
        match self {
            F32 => {
                let bytes = [bytes[0], bytes[1], bytes[2], bytes[3]];
                f64::from(f32::from_bits(u32::from_be_bytes(bytes)))
            },
            F64 => f64::from_bits(u64::from_be_bytes(bytes)),
        }
    }
    fn get_float<R: Read>(&self, r: &mut R) -> Result<f64> {
        let mut bytes = [0u8; 8];
        let len = self.get_size();
        r.read_exact(&mut bytes[..len])?;
        Ok(self.parse(bytes))
    }
}

pub fn format_float(kind: Kind, mut data: Box<dyn Read>, output: StdoutLock) -> Result<()> {
    use std::io::ErrorKind::*;
    let mut buffer = String::new();
    loop {
        let float: f64 = match kind.get_float(&mut data) {
            Ok(f) => f,
            Err(e) => match e.kind() {
                UnexpectedEof => return Ok(()),
                _ => return Err(e),
            },
        };
        buffer.clear();
        writeln!(buffer, "{:.6}", float).unwrap();
        if let Err(e) = output.write_all(buffer.as_bytes()) {
            match e.kind() {
                BrokenPipe => return Ok(()),
                _ => return Err(e),
            }
        }
    }
}
