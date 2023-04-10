use candid::CandidType;

#[derive(CandidType)]
pub enum UpdateStateResponse{
    Success,
    Unauthorized,
}

#[derive(CandidType)]
pub enum RegistrationResponse{
    Success{ username: String },
    UserNameTaken,
    AlreadyRegistered,
    VerificationAmountNotPaid,
    FirstNameTooLong,
    MiddleNameTooLong,
    LastNameTooLong,
    FailedToAllocateMemory,
}

#[derive(CandidType)]
pub enum CreatePostResponse{
    Success,
    UserNotRegistered,
    TopicTooLong,
    MessageTooLong,
    FailedToAllocateMemory,
}