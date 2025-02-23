#[derive(Debug)]
pub enum CompileError {
    DirectoryNotFound,
    CompileCommandError(String),
    TokioError(tokio::io::Error),
    ZipError(async_zip::error::ZipError),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::DirectoryNotFound => write!(f, "Directory not found"),
            CompileError::CompileCommandError(e) => write!(f, "Compile command error: {}", e),
            CompileError::TokioError(e) => e.fmt(f),
            CompileError::ZipError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<tokio::io::Error> for CompileError {
    fn from(e: tokio::io::Error) -> Self {
        CompileError::TokioError(e)
    }
}

impl From<async_zip::error::ZipError> for CompileError {
    fn from(e: async_zip::error::ZipError) -> Self {
        CompileError::ZipError(e)
    }
}

#[derive(Debug)]
pub enum DecompileError {
    TokioError(tokio::io::Error),
}

impl std::fmt::Display for DecompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecompileError::TokioError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for DecompileError {}

impl From<tokio::io::Error> for DecompileError {
    fn from(e: tokio::io::Error) -> Self {
        DecompileError::TokioError(e)
    }
}
