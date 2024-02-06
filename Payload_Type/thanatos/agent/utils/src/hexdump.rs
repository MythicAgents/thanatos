//! Utils for printing out hexdumps

use std::ops::Add;

/// Format for printing out the hexdump
#[derive(Default)]
pub enum HexdumpFormat {
    /// Hexdump in compact format
    #[default]
    Compact,

    /// Hexdump in xxd format
    Xxd,

    /// Hexdump in xxd format with colors
    XxdColored,
}

trait HexdumpFormatter {
    fn offset_string(&self, idx: usize) -> String;
    fn hexdump_bytes(&self, data: &[u8]) -> String;
    fn ascii_table_string(&self, data: &[u8]) -> String;
}

struct CompactFormatter {
    zero_padding: usize,
}

impl HexdumpFormatter for CompactFormatter {
    fn offset_string(&self, idx: usize) -> String {
        let zero_pad = self.zero_padding;
        format!("0x{:0>zero_pad$x}|", idx).to_string()
    }

    fn hexdump_bytes(&self, data: &[u8]) -> String {
        let mut output = String::new();
        for word in data.chunks(2) {
            for b in word {
                output.push_str(&format!("{:0>2x}", b));
            }

            output += " ";
        }

        output
    }

    fn ascii_table_string(&self, data: &[u8]) -> String {
        let ascii: String = data
            .iter()
            .map(|b| {
                let c = char::from(*b);
                if (' '..='~').contains(&c) {
                    c
                } else {
                    '.'
                }
            })
            .collect();

        format!("|{ascii}").to_string()
    }
}

struct XxdFormatter;
impl HexdumpFormatter for XxdFormatter {
    fn offset_string(&self, idx: usize) -> String {
        format!("{:0>8x}:", idx).to_string()
    }

    fn hexdump_bytes(&self, data: &[u8]) -> String {
        let mut output = String::new();
        for word in data.chunks(2) {
            for b in word {
                output.push_str(&format!("{:0>2x}", b));
            }

            output += " ";
        }

        output
    }

    fn ascii_table_string(&self, data: &[u8]) -> String {
        let ascii: String = data
            .iter()
            .map(|b| {
                let c = char::from(*b);
                if (' '..='~').contains(&c) {
                    c
                } else {
                    '.'
                }
            })
            .collect();

        format!(" {ascii}").to_string()
    }
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

struct XxdColoredFormatter;
impl HexdumpFormatter for XxdColoredFormatter {
    fn offset_string(&self, idx: usize) -> String {
        format!("{:0>8x}:", idx).to_string()
    }

    fn hexdump_bytes(&self, data: &[u8]) -> String {
        let mut output = String::new();
        for word in data.chunks(2) {
            for b in word.iter() {
                if *b == 0x9 || *b == 0xa || *b == 0xd {
                    output.push_str(&format!(
                        "{}{:0>2x}{}",
                        AnsiColor::Yellow.as_str(),
                        b,
                        AnsiColor::Reset.as_str()
                    ));
                } else if (*b >= 0x1 && *b <= 0x1f) || *b >= 0x7f {
                    output.push_str(&format!(
                        "{}{:0>2x}{}",
                        AnsiColor::Red.as_str(),
                        b,
                        AnsiColor::Reset.as_str(),
                    ));
                } else if *b >= 0x20 && *b <= 0x7e {
                    output.push_str(&format!(
                        "{}{:0>2x}{}",
                        AnsiColor::Green.as_str(),
                        b,
                        AnsiColor::Reset.as_str(),
                    ));
                } else {
                    output.push_str(&format!("{:0>2x}", b));
                }
            }

            output += " ";
        }

        output
    }

    fn ascii_table_string(&self, data: &[u8]) -> String {
        let mut output = String::new();
        for b in data {
            let c = char::from(*b);
            if *b == 0x9 || *b == 0xa || *b == 0xd {
                output.push_str(&format!(
                    "{}.{}",
                    AnsiColor::Yellow.as_str(),
                    AnsiColor::Reset.as_str()
                ));
            } else if (*b >= 0x1 && *b <= 0x1f) || *b >= 0x7f {
                output.push_str(&format!(
                    "{}.{}",
                    AnsiColor::Red.as_str(),
                    AnsiColor::Reset.as_str(),
                ));
            } else if *b >= 0x20 && *b <= 0x7e {
                output.push_str(&format!(
                    "{}{}{}",
                    AnsiColor::Green.as_str(),
                    c,
                    AnsiColor::Reset.as_str(),
                ));
            } else {
                output.push('.');
            }
        }

        format!(" {output}").to_string()
    }
}

/// Prints out a hexdump of some data
pub fn hexdump(data: impl AsRef<[u8]>, format: HexdumpFormat) {
    let data = data.as_ref();

    let formatter: Box<dyn HexdumpFormatter> = match format {
        HexdumpFormat::Compact => Box::new(CompactFormatter {
            zero_padding: (data.as_ref().len() as f32).log10().add(1.).trunc() as usize,
        }),
        HexdumpFormat::Xxd => Box::new(XxdFormatter),
        HexdumpFormat::XxdColored => Box::new(XxdColoredFormatter),
    };

    for (idx, line) in data.chunks(0x10).enumerate() {
        print!("{} ", formatter.offset_string(idx * 0x10));
        print!("{}", formatter.hexdump_bytes(line));

        let mut remaining = 16 - line.len();
        if line.len() % 2 == 1 {
            print!("  ");
            remaining -= 1;
        }

        let pad = (remaining * 2) + (remaining / 2);
        for _ in 0..pad {
            print!(" ");
        }

        println!("{}", formatter.ascii_table_string(line));
    }
}
