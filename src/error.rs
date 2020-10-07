#[derive(Debug)]
pub enum ControllerError {
    InterruptsEnabled,
    Timeout,
    TestFailed { response: u8 },
}
