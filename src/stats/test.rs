use super::Histogram;
use crate::stats::Bucket;

#[test]
fn can_add_samples() {
    let mut hist = Histogram::new(&vec![10, 20, 30]);
    assert_eq!(hist.buckets(), 4);

    hist.add(5);
    hist.add(7);
    hist.add(20);
    hist.add(15);
    hist.add(21);
    hist.add(25);
    hist.add(40);

    assert_eq!(hist.count(), 7);
    assert_eq!(hist.sum(), 133);

    assert_eq!(
        hist.get_bucket(0),
        Some(&Bucket {
            count: 2,
            sum: 7 + 5,
            ceiling: 10
        })
    );
    assert_eq!(
        hist.get_bucket(1),
        Some(&Bucket {
            count: 2,
            sum: 35,
            ceiling: 20
        })
    );
    assert_eq!(
        hist.get_bucket(2),
        Some(&Bucket {
            count: 2,
            sum: 46,
            ceiling: 30
        })
    );
    assert_eq!(
        hist.get_bucket(3),
        Some(&Bucket {
            count: 1,
            sum: 40,
            ceiling: u64::MAX
        })
    );
}
