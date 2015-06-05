pub enum ErrorCode {
    NotDefined                 = 0,
    FileNotFound               = 1,
    AccessViolation            = 2,
    DiskFullOrAllocationFailed = 3,
    IllegalTftpOperation       = 4,
    UnknownTransferId          = 5,
    FileAlreadyExists          = 6,
    NoSuchUser                 = 7
}

impl ErrorCode {
    pub fn from_u16(val : u16) -> ErrorCode {
        match val {
            0 => ErrorCode::NotDefined,
            1 => ErrorCode::FileNotFound,
            2 => ErrorCode::AccessViolation,
            3 => ErrorCode::DiskFullOrAllocationFailed,
            4 => ErrorCode::IllegalTftpOperation,
            5 => ErrorCode::UnknownTransferId,
            6 => ErrorCode::FileAlreadyExists,
            7 => ErrorCode::NoSuchUser,
            _ => ErrorCode::NotDefined,
        }
    }
}

pub enum Packet {
    RRQ(String, String),
    WRQ(String, String),
    ACK(u16),
    ERROR(ErrorCode, String),
    Data(u16, Box<Vec<u8>>),
}

