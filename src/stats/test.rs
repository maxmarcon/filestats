use super::Histogram;
use crate::stats::Bucket;

#[test]
fn can_add_samples() {
    let mut hist = Histogram::new(&[10, 20, 30]);

    hist.add(5);
    hist.add(7);
    hist.add(20);
    hist.add(15);
    hist.add(21);
    hist.add(25);
    hist.add(40);

    assert_eq!(hist.count(), 7);
    assert_eq!(hist.sum(), 133);

    let expected_buckets = vec![
        Bucket {
            count: 2,
            sum: 7 + 5,
            ceiling: 10,
        },
        Bucket {
            count: 2,
            sum: 35,
            ceiling: 20,
        },
        Bucket {
            count: 2,
            sum: 46,
            ceiling: 30,
        },
        Bucket {
            count: 1,
            sum: 40,
            ceiling: u64::MAX,
        },
    ];

    assert_eq!(hist.buckets(), expected_buckets);
}

#[test]
fn can_deal_with_u64_max_as_top_bucket() {
    let hist = Histogram::new(&vec![10, u64::MAX]);

    let expected_buckets = vec![
        Bucket {
            count: 0,
            sum: 0,
            ceiling: 10,
        },
        Bucket {
            count: 0,
            sum: 0,
            ceiling: u64::MAX,
        },
    ];

    assert_eq!(hist.buckets(), expected_buckets);
}
