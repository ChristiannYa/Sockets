/// An 8-bit sequence number that compares in a circular (wrapping) number
/// space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Seq(pub u8);

impl Seq {
    /// Returns `true` if `self` is older than `other`
    pub fn is_newer_than(self, other: Seq) -> bool {
        // `self.0.wrapping_sub(other.0)` is the forward distance from `other.0`
        // to `self.0` on the 0..255 range.
        //
        // A distance of 1-127 means `self.0` is ahead of `other.0`, so it's
        // newer.
        // A distance of 128-255 means it's behind, so it's stale.
        let diff = self.0.wrapping_sub(other.0);
        diff != 0 && diff < 128
    }
}
