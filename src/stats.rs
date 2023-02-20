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
            Color::Magenta,
            Color::Red,
        ];

        let total = self.count();
        let max = self.mode().unwrap().count;

        let padding = ((100.0 * (max as f32 / total as f32)) / PERC_POINT_PER_BAR as f32) as usize;

        while let Some(bucket) = iter.next() {
            if bucket.count > 0 {
                let base = iter.peek().map_or(0, |&b| b.ceiling);

                let perc = 100.0 * (bucket.count as f32 / total as f32);

                let (histogram, color) = hist_bars(perc, &colors, PERC_POINT_PER_BAR);
                write!(
                    f,
                    "{} {} {} {:7$} {:>5.1}{} {}\n",
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

    format!("{:<7}", byte_string)
}

fn hist_bars(perc: f32, colors: &[Color], perc_point_per_bar: u32) -> (String, Color) {
    if perc > 100.0 || perc < 0.0 {
        panic!("perc should be a percentage, got {}", perc);
    }

    let bars = (perc / perc_point_per_bar as f32) as usize;
    let bars_per_colors = (100 as f32 / PERC_POINT_PER_BAR as f32) as usize / colors.len() as usize;
    let mut bar_colors = colors.iter().cycle();
    let mut str = String::new();

    let mut current_color = colors[0];
    for _ in 0..(bars / bars_per_colors as usize) {
        current_color = *bar_colors.next().unwrap();
        str += &format!(
            "{}",
            style("|".repeat(bars_per_colors as usize)).fg(current_color)
        );
    }

    let remainder = bars % bars_per_colors as usize;
    if remainder > 0 {
        current_color = *bar_colors.next().unwrap();
    }

    str += &format!("{}", style("|".repeat(remainder)).fg(current_color));

    (str, current_color)
}
