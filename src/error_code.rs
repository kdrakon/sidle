pub type ErrorCode = u8;

pub const ABORT: u8 = 1;

pub const KEY_INPUT_ERROR: u8 = 100;
pub const COULD_NOT_LIST_DIR: u8 = 101;
pub const COULD_NOT_READ_METADATA: u8 = 102;
pub const ERROR_WRITING_TO_OUTPUT: u8 = 103;

// UI-related
pub const FAILED_TO_FLUSH_UI_SCREEN: u8 = 201;
pub const FAILED_TO_WRITE_TO_UI_SCREEN: u8 = 202;
pub const FAILED_TO_CREATE_UI_SCREEN: u8 = 203;
pub const COULD_NOT_DETERMINE_TERMINAL_SIZE: u8 = 204;
