use core::fmt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct Signature<const N: usize>([u8; N]);

impl<const N: usize> Signature<N> {
    /// Creates a new signature from the given ascii character array. Invalid characters are left as 0.
    pub(crate) const fn new_lossy(val: [char; N]) -> Self {
        let mut array = [0u8; N];
        let mut i = 0;
        while i < N {
            array[i] = (val[i] as u32 & 0x7F) as u8;
            i += 1;
        }
        Signature(array)
    }
}
impl<const N: usize> fmt::Debug for Signature<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let converted: [char; N] = (*self).into();
        write!(f, "{:?}", converted)
    }
}

impl<const N: usize> fmt::Display for Signature<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let converted: [char; N] = (*self).into();
        write!(f, "{:?}", converted)
    }
}

impl<const N: usize> From<Signature<N>> for [char; N] {
    fn from(value: Signature<N>) -> Self {
        value.0.map(|byte| byte as char)
    }
}
