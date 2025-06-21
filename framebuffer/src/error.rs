#[derive(Debug, thiserror::Error)]
pub enum FrameBufferError {
    #[error("Coordinates out of bounds: x={0}, y={1}")]
    CoordinatesOutOfBounds(usize, usize),
    #[error("Unsupported font character")]
    InvalidCharacter,
}
