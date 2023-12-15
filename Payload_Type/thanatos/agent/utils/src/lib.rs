//! Helper utilities

use std::ops::Add;
pub mod uuid;

/// Prints out a hexdump of some data
pub fn hexdump(data: impl AsRef<[u8]>) {
    let data = data.as_ref();
    let zero_pad = (data.len() as f32).log10().add(1.).trunc() as usize;

    for (idx, line) in data.chunks(0x10).enumerate() {
        print!("0x{:0>zero_pad$x}| ", idx * 0x10);
        for word in line.chunks(2) {
            for b in word {
                print!("{:0>2x}", b);
            }

            print!(" ");
        }

        let ascii: String = line
            .iter()
            .map(|b| {
                let c = char::from(*b);
                (c >= ' ' && c <= '~').then_some(c).unwrap_or('.')
            })
            .collect();

        if line.len() == 16 {
            println!("|{ascii}");
            continue;
        }

        let mut remaining = 16 - line.len();
        if line.len() % 2 == 1 {
            print!("  ");
            remaining -= 1;
        }

        let pad = (remaining * 2) + (remaining / 2);
        for _ in 0..pad {
            print!(" ");
        }

        println!("|{ascii}");
    }
}
