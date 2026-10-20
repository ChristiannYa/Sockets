/// Split a records buffer, `[record_count, (len, bytes)*]`, into its records.
/// Stops early on a truncated buffer instead of panicking.
pub fn split(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    let (count, mut rest) = match bytes.split_first() {
        Some((&count, rest)) => (count, rest),
        None => (0, bytes),
    };

    (0..count).map_while(move |_| {
        let (&len, rem) = rest.split_first()?;
        let len = len as usize;
        if rem.len() < len {
            return None;
        }
        let (record, rem) = rem.split_at(len);
        rest = rem;
        Some(record)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_records() {
        let buf = [2, 2, 10, 11, 1, 20];
        let got: Vec<&[u8]> = split(&buf).collect();
        assert_eq!(got, vec![&[10, 11][..], &[20][..]]);
    }

    #[test]
    fn stops_on_truncated_buffer() {
        let buf = [2, 2, 10, 11, 5, 20];
        assert_eq!(split(&buf).count(), 1);
    }

    #[test]
    fn empty_buffer_has_no_records() {
        assert_eq!(split(&[]).count(), 0);
    }
}
