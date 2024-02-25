//! Utils for printing out hexdumps

use std::io::Write;

#[macro_export]
macro_rules! hexdump {
    ($dat:ident) => {
        let _ = $crate::hexdump::hexdump_writer(&mut std::io::stdout(), $dat);
    };
}

enum AnsiColor {
    Reset,
    Red,
    Green,
    Yellow,
}

impl AnsiColor {
    fn as_str(&self) -> &'static str {
        match self {
            AnsiColor::Reset => "\x1b[0m",
            AnsiColor::Red => "\x1b[0;31m",
            AnsiColor::Green => "\x1b[0;32m",
            AnsiColor::Yellow => "\x1b[0;33m",
        }
    }
}

/// Prints out a hexdump of some data
pub fn hexdump_writer(output: &mut impl Write, data: impl AsRef<[u8]>) -> std::io::Result<()> {
    let data = data.as_ref();

    for (idx, line) in data.chunks(0x10).enumerate() {
        write!(output, "{:0>8x}: ", idx * 0x10)?;

        for word in line.chunks(2) {
            for b in word.iter() {
                if *b == 0x9 || *b == 0xa || *b == 0xd {
                    write!(
                        output,
                        "{}{:0>2x}{}",
                        AnsiColor::Yellow.as_str(),
                        b,
                        AnsiColor::Reset.as_str(),
                    )?;
                } else if (*b >= 0x1 && *b <= 0x1f) || *b >= 0x7f {
                    write!(
                        output,
                        "{}{:0>2x}{}",
                        AnsiColor::Red.as_str(),
                        b,
                        AnsiColor::Reset.as_str(),
                    )?;
                } else if *b >= 0x20 && *b <= 0x7e {
                    write!(
                        output,
                        "{}{:0>2x}{}",
                        AnsiColor::Green.as_str(),
                        b,
                        AnsiColor::Reset.as_str(),
                    )?;
                } else {
                    write!(output, "{:0>2x}", b)?;
                }
            }

            write!(output, " ")?;
        }

        let mut remaining = 16 - line.len();
        if line.len() % 2 == 1 {
            write!(output, "  ")?;
            remaining -= 1;
        }

        let pad = (remaining * 2) + (remaining / 2);
        for _ in 0..pad {
            write!(output, " ")?;
        }

        for b in line {
            let c = char::from(*b);
            if *b == 0x9 || *b == 0xa || *b == 0xd {
                write!(
                    output,
                    "{}.{}",
                    AnsiColor::Yellow.as_str(),
                    AnsiColor::Reset.as_str()
                )?;
            } else if (*b >= 0x1 && *b <= 0x1f) || *b >= 0x7f {
                write!(
                    output,
                    "{}.{}",
                    AnsiColor::Red.as_str(),
                    AnsiColor::Reset.as_str(),
                )?;
            } else if *b >= 0x20 && *b <= 0x7e {
                write!(
                    output,
                    "{}{}{}",
                    AnsiColor::Green.as_str(),
                    c,
                    AnsiColor::Reset.as_str(),
                )?;
            } else {
                write!(output, ".")?;
            }
        }
        writeln!(output)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_test() {
        let mut output = Vec::new();
        let data = (0..=0xff).collect::<Vec<u8>>();
        super::hexdump_writer(&mut output, data).unwrap();
    }
}
