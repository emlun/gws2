pub type ExitCode = i32;

pub const OK: ExitCode = 0;
pub const UNKNOWN_ERROR: ExitCode = 1;
pub const NO_PROJECTS_FILE: ExitCode = 2;
pub const BAD_PROJECTS_FILE: ExitCode = 3;

pub const STATUS_PROJECT_FAILED: ExitCode = 4;
