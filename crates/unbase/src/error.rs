#[derive(PartialEq, Debug)]
pub enum RetrieveError {
    NotFound,
    NotFoundByDeadline,
    AccessDenied,
    InvalidHead(InvalidHead),
    IndexNotInitialized,
    SlabError,
    MemoLineageError,
    WriteError(Box<WriteError>),
}

#[derive(PartialEq, Debug)]
pub enum InvalidHead {
    MissingEntityId,
    Empty,
}

#[derive(PartialEq, Debug)]
pub enum WriteError {
    Unknown,
    RetrieveError(Box<RetrieveError>),
    // This is silly. TODO - break this cycle and remove the Box
    BadTarget,
}

#[derive(PartialEq, Debug)]
pub enum ObserveError {
    Unknown,
}

impl core::convert::From<()> for ObserveError {
    fn from(_error: ()) -> Self {
        ObserveError::Unknown
    }
}
impl core::convert::From<RetrieveError> for WriteError {
    fn from(error: RetrieveError) -> Self {
        WriteError::RetrieveError(Box::new(error))
    }
}
impl core::convert::From<WriteError> for RetrieveError {
    fn from(error: WriteError) -> Self {
        RetrieveError::WriteError(Box::new(error))
    }
}

#[derive(PartialEq, Debug)]
pub enum StorageOpDeclined {
    InsufficientPeering,
}

#[derive(PartialEq, Debug)]
pub enum PeeringError {
    InsufficientPeering,
}
