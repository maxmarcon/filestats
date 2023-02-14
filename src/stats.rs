#![allow(dead_code)]

#[cfg(test)]
mod test;

use console::{pad_str, style, Alignment, Color};

#[derive(PartialEq, Debug)]
pub struct Bucket {
    count: u32,
    sum: u64,
    ceiling: u64,
}

#[derive(Debug)]
pub struct Histogram {
    buckets: Vec<Bucket>,
}

impl Histogram {
    pub fn new(limits: &[u64]) -> Self {
        let mut buckets: Vec<Bucket> = limits
            .iter()
            .filter(|&limit| *limit != u64::MAX)
            .map(|limit| Bucket {
                count: 0,
                sum: 0,
                ceiling: *limit,
            })
            .collect();

        buckets.push(Bucket {
            count: 0,
            sum: 0,
            ceiling: u64::MAX,
        });

        Histogram { buckets: buckets }
    }

    pub fn add(&mut self, sample: u64) {
        for mut bucket in self.buckets.iter_mut() {
            if sample <= bucket.ceiling {
                bucket.count += 1;
                bucket.sum += sample;
                break;
            }
        }
    }

    pub fn count(&self) -> u32 {
        self.buckets.iter().map(|bucket| bucket.count).sum()
    }

    pub fn sum(&self) -> u64 {
        self.buckets.iter().map(|bucket| bucket.sum).sum()
    }

    pub fn buckets(&self) -> &[Bucket] {
        self.buckets.as_slice()
    }
}

impl std::fmt::Display for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.buckets().iter().rev().peekable();

        let mut colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
        ]
        .iter()
        .cycle();

        while let Some(bucket) = iter.next() {
            if bucket.count > 0 {
                let base = iter.peek().map_or(0, |&b| b.ceiling);
                let color = *colors.next().unwrap();

                let bucket_repr = format!(
                    "{} to {} = {}\n",
                    format_bytes(base),
                    format_bytes(bucket.ceiling),
                    bucket.count
                );

                write!(f, "{}", style(bucket_repr).fg(color))?
            }
        }

        Ok(())
    }
}

fn format_bytes(size: u64) -> String {
    const UNIT_SIZES: [u64; 3] = [2_u64.pow(30), 2_u64.pow(20), 2_u64.pow(10)];
    const UNIT_NAMES: [char; 3] = ['G', 'M', 'K'];

    let mut byte_string = None;

    for (&unit_size, unit_name) in UNIT_SIZES.iter().zip(UNIT_NAMES) {
        if size >= unit_size {
            byte_string = Some(format!("{}{}iB", size / unit_size, unit_name));
            break;
        }
    }

    let byte_string = byte_string.unwrap_or(format!("{}B", size));

    pad_str(&byte_string, 7, Alignment::Left, None).into_owned()
}
