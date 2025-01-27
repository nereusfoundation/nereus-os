use core::fmt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct Signature<const N: usize>(pub(crate) [u8; N]);

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

impl<const N: usize> PartialEq<[u8; N]> for Signature<N> {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.0 == *other
    }
}
