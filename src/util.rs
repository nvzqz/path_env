use std::ffi::OsString;

/// An approximate length for how much to reserve when appending multiple paths.
///
/// This serves to reduce the number of allocations necessary.
const LEN_HEURISTIC: usize = ":/usr/local/bin".len();

#[inline(always)]
pub fn reserve_heuristic(path: &mut OsString, old_len: usize) {
    let len = old_len.saturating_mul(LEN_HEURISTIC);
    let len = if len >= (isize::max_value() / 2) as usize {
        // Avoid false failure if the heuristic is somehow totally wrong.
        old_len
    } else {
        len
    };
    path.reserve(len);
}
