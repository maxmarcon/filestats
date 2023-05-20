#[cfg(test)]
mod test;

use crate::utils::format_bytes;
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

        Histogram { buckets }
    }

    pub fn add(&mut self, sample: u64) -> &mut Self {
        for mut bucket in self.buckets.iter_mut() {
            if sample <= bucket.ceiling {
                bucket.count += 1;
                bucket.sum += sample;
                break;
            }
        }
        self
    }

    pub fn count(&self) -> u32 {
        self.buckets.iter().map(|bucket| bucket.count).sum()
    }

    pub fn sum(&self) -> u64 {
        self.buckets.iter().map(|bucket| bucket.sum).sum()
    }

    pub fn avg(&self) -> Option<f64> {
        if self.count() > 0 {
            Some(self.sum() as f64 / self.count() as f64)
        } else {
            None
        }
    }

    pub fn buckets(&self) -> &[Bucket] {
        self.buckets.as_slice()
    }

    pub fn mode(&self) -> Option<&Bucket> {
        self.buckets
            .iter()
            .max_by(|&b1, &b2| b1.count.cmp(&b2.count))
    }
}

const PERC_POINT_PER_BAR: u32 = 2;

impl std::fmt::Display for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.buckets().iter().rev().peekable();

        let colors = [
            Color::White,
            Color::Yellow,
            Color::Green,
            Color::Blue,
            Color::Red,
        ];

        let total = self.count();
        let max_bucket = self.mode().unwrap().count;

        let padding =
            ((100.0 * (max_bucket as f32 / total as f32)) / PERC_POINT_PER_BAR as f32) as usize;

        while let Some(bucket) = iter.next() {
            if bucket.count > 0 {
                let base = iter.peek().map_or(0, |&b| b.ceiling);

                let perc = 100.0 * (bucket.count as f32 / total as f32);

                let (histogram, color) = hist_bars(perc, &colors, PERC_POINT_PER_BAR);
                writeln!(
                    f,
                    "{:<7} {} {:<7} {:7$} {:>5.1}{} {}",
                    style(format_bytes(base)).fg(color),
                    style("to").fg(color),
                    style(format_bytes(bucket.ceiling)).fg(color),
                    pad_str(&histogram, padding, Alignment::Left, None),
                    style(perc).fg(color),
                    style("%").fg(color),
                    style(bucket.count).fg(color),
                    padding
                )?;
            }
        }

        Ok(())
    }
}

fn hist_bars(perc: f32, colors: &[Color], perc_point_per_bar: u32) -> (String, Color) {
    if !(0.0..=100.0).contains(&perc) {
        panic!("perc should be a percentage, got {}", perc);
    }

    let bars = (perc / perc_point_per_bar as f32) as usize;
    let bars_per_colors = (100_f32 / PERC_POINT_PER_BAR as f32) as usize / colors.len();
    let mut bar_colors = colors.iter().cycle();
    let mut str = String::new();

    let mut current_color = colors[0];
    for _ in 0..(bars / bars_per_colors) {
        current_color = *bar_colors.next().unwrap();
        str += &format!("{}", style("|".repeat(bars_per_colors)).fg(current_color));
    }

    let remainder = bars % bars_per_colors;
    if remainder > 0 {
        current_color = *bar_colors.next().unwrap();
    }

    str += &format!("{}", style("|".repeat(remainder)).fg(current_color));

    (str, current_color)
}
