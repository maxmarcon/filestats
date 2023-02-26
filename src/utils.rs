pub fn format_bytes(size: u64) -> String {
    const UNIT_SIZES: [u64; 3] = [2_u64.pow(30), 2_u64.pow(20), 2_u64.pow(10)];
    const UNIT_NAMES: [char; 3] = ['G', 'M', 'K'];

    let mut byte_string = None;

    for (&unit_size, unit_name) in UNIT_SIZES.iter().zip(UNIT_NAMES) {
        if size >= unit_size {
            byte_string = Some(format!("{}{}iB", size / unit_size, unit_name));
            break;
        }
    }

    byte_string.unwrap_or(format!("{}B", size))
}
