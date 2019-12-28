#[derive(PartialEq, Debug)]
pub enum RetrieveError {
    NotFound,
    NotFoundByDeadline,
    AccessDenied,
    InvalidMemoRefHead,
    IndexNotInitialized,
    SlabError
}

#[derive(PartialEq, Debug)]
pub enum PeeringError {
    InsufficientReplicas
}