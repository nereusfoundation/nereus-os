use super::signature::Signature;

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum AcpiError {
    #[error("Invalid RSD address")]
    RsdAddress,
    #[error("Invalid XSDT address")]
    RsdtAddress,
    #[error("Table not found: {0}")]
    TableNotFound(Signature<4>),
}
