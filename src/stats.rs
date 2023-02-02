#[cfg(test)]
mod test;

struct Bucket {
    count: u32,
    size_sum: u64,
    max_size: u64,
}

struct Histogram {
    buckets: Vec<Bucket>,
}

impl Histogram {
    fn new(limits: &[u64]) -> Self {
        let buckets = limits
            .iter()
            .map(|limit| Bucket {
                count: 0,
                size_sum: 0,
                max_size: *limit,
            })
            .collect();

        Histogram { buckets: buckets }
    }

    fn add(&mut self, size: u64) {}

    fn count(&self) -> u32 {}

    fn sum(&self) -> u64 {
        0
    }
}
