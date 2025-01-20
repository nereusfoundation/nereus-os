#[cfg(feature = "bump")]
pub mod bump;
#[cfg(feature = "linked-list")]
pub mod linked_list;

/// Aligns a given number to the specified alignment.
pub fn align_up(number: u64, align: usize) -> u64 {
    let align = align as u64;
    (number + align - 1) & !(align - 1)
}
