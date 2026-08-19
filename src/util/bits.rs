pub fn mask(bits_len: &usize) -> u32 { (1 << bits_len) - 1 }

pub fn split_into_bytes(s: &str) -> String {
    let len = s.len();
    let rem = len % 8;
    let mut groups = Vec::new();

    let mut i = 0;
    if rem != 0 {
        groups.push(&s[0..rem]);
        i = rem;
    }
    while i < len {
        groups.push(&s[i..i + 8]);
        i += 8;
    }

    groups.join(" ")
}
