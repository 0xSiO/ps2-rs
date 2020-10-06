#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    BufferOverrun,
    SelfTestPassed,
    Echo,
    Acknowledged,
    SelfTestFailed,
    Resend,
    KeyDetectionError,
    Scancode(u8),
}

impl From<u8> for Response {
    fn from(byte: u8) -> Self {
        use Response::*;
        match byte {
            0x00 => BufferOverrun,
            0xaa => SelfTestPassed,
            0xee => Echo,
            0xfa => Acknowledged,
            0xfc => SelfTestFailed,
            0xfe => Resend,
            0xff => KeyDetectionError,
            other => Scancode(other),
        }
    }
}
