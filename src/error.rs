use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    IndexOutOfBounds { index: usize, max: usize },
    StageFailed(&'static str),
    Custom(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::IndexOutOfBounds { index, max } => {
                write!(f, "Index {} is out of bounds (max: {})", index, max)
            }
            PipelineError::StageFailed(msg) => write!(f, "Stage processing failed: {}", msg),
            PipelineError::Custom(msg) => write!(f, "Pipeline error: {}", msg),
        }
    }
}

impl std::error::Error for PipelineError {}