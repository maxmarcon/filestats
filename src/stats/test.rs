use super::Histogram;

#[test]
fn can_add_bucket() {
    let mut hist = Histogram::new(&vec![10, 20, 30]);

    hist.add(5);
    hist.add(20);
    hist.add(25);
    hist.add(40);

    assert_eq!(hist.count(), 4);
    assert_eq!(hist.sum(), 90);
}
