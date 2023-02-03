#[cfg(test)]
mod test;

#[derive(PartialEq, Debug)]
struct Bucket {
    count: u32,
    sum: u64,
    ceiling: u64,
}

struct Histogram {
    buckets: Vec<Bucket>,
}

impl Histogram {
    fn new(limits: &[u64]) -> Self {
        let mut buckets: Vec<Bucket> = limits
            .iter()
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

    fn add(&mut self, sample: u64) {
        for mut bucket in self.buckets.iter_mut() {
            if sample <= bucket.ceiling {
                bucket.count += 1;
                bucket.sum += sample;
                break;
            }
        }
    }

    fn count(&self) -> u32 {
        self.buckets.iter().map(|bucket| bucket.count).sum()
    }

    fn sum(&self) -> u64 {
        self.buckets.iter().map(|bucket| bucket.sum).sum()
    }

    fn buckets(&self) -> usize {
        self.buckets.len()
    }

    fn get_bucket(&self, index: usize) -> Option<&Bucket> {
        self.buckets.get(index)
    }
}
