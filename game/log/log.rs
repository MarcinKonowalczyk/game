#[cfg(feature = "native")]
use crate::log::vsprintf::*;

use crate::webhacks;

#[allow(unused)]
pub const ALL: i32 = 0;

pub const TRACE: i32 = 1;
pub const DEBUG: i32 = 2;
pub const INFO: i32 = 3;
pub const WARNING: i32 = 4;
pub const ERROR: i32 = 5;
pub const FATAL: i32 = 6;
pub const NONE: i32 = 999;

#[cfg(feature = "native")]
mod color {
    pub const GREEN: &str = "\x1b[92m";
    pub const YELLOW: &str = "\x1b[93m";
    pub const RED: &str = "\x1b[91m";
    pub const CYAN: &str = "\x1b[96m";
    pub const MAGENTA: &str = "\x1b[95m";
    pub const RESET: &str = "\x1b[0m";
}

#[cfg(feature = "web")]
mod color {
    pub const GREEN: &str = "color: green;";
    pub const YELLOW: &str = "color: yellow;";
    pub const RED: &str = "color: red;";
    pub const CYAN: &str = "color: cyan;";
    pub const MAGENTA: &str = "color: magenta;";
    pub const RESET: &str = "";
}

// Function to plug into raylibs SetTraceLogCallback
#[cfg(feature = "native")]
#[no_mangle]
pub unsafe extern "C" fn native_log_callback(log_level: i32, text: *const i8, _args: *mut VaList) {
    let text = std::ffi::CStr::from_ptr(text);
    let text = text.to_str().unwrap();

    let text = match log_level {
        INFO => format!("[{}INFO{}] : {}", color::GREEN, color::RESET, text),
        ERROR => format!("[{}ERROR{}] : {}", color::RED, color::RESET, text),
        FATAL => format!("[{}FATAL{}] : {}", color::RED, color::RESET, text),
        WARNING => format!("[{}WARN{}] : {}", color::YELLOW, color::RESET, text),
        DEBUG => format!("[{}DEBUG{}] : {}", color::CYAN, color::RESET, text),
        TRACE => format!("[{}TRACE{}] : {}", color::MAGENTA, color::RESET, text),
        NONE => format!("{}", text),
        _ => format!("{}", text),
    };

    // call _vprintf directly
    // vprintf(&*(text + "\n"), _args);

    // call _vsnprintf to format the string first
    println!("{}", vsnprintf(&text, _args));
}

#[cfg(feature = "web")]
#[no_mangle]
pub fn web_log_callback(log_level: i32, text_ptr: *const i8) {
    // read text from the pointer
    let text = unsafe { std::ffi::CStr::from_ptr(text_ptr) };
    let text = text.to_str().unwrap();

    let color = match log_level {
        INFO => color::GREEN,
        ERROR => color::RED,
        FATAL => color::RED,
        WARNING => color::YELLOW,
        DEBUG => color::CYAN,
        TRACE => color::MAGENTA,
        NONE => "",
        _ => "",
    };
    let text = match log_level {
        INFO => format!("[%c INFO %c] : {}", text),
        ERROR => format!("[%c ERROR %c] : {}", text),
        FATAL => format!("[%c FATAL %c] : {}", text),
        WARNING => format!("[%c WARN %c] : {}", text),
        DEBUG => format!("[%c DEBUG %c] : {}", text),
        TRACE => format!("[%c TRACE %c] : {}", text),
        NONE => format!("{}", text),
        _ => format!("{}", text),
    };

    if color.is_empty() {
        webhacks::_console_log_args(text.as_str(), None);
    } else {
        webhacks::_console_log_args(text.as_str(), Some(vec![color, color::RESET]));
    }
}

pub fn set_trace_log_callback() {
    #[cfg(feature = "native")]
    let callback = Some(native_log_callback as webhacks::LogCallback);
    #[cfg(feature = "web")]
    let callback = Some(web_log_callback as webhacks::LogCallback);

    webhacks::set_trace_log_callback(callback, "web_log_callback");
}

#[allow(unused)]
pub fn set_log_level(level: i32) {
    webhacks::set_log_level(level);
}

#[allow(unused)]
pub fn info(text: &str) {
    webhacks::log(INFO, text);
}

#[allow(unused)]
pub fn error(text: &str) {
    webhacks::log(ERROR, text);
}

#[allow(unused)]
pub fn warning(text: &str) {
    webhacks::log(WARNING, text);
}

#[allow(unused)]
pub fn debug(text: &str) {
    webhacks::log(DEBUG, text);
}

#[allow(unused)]
pub fn trace(text: &str) {
    webhacks::log(TRACE, text);
}

#[allow(unused)]
pub fn fatal(text: &str) {
    webhacks::log(FATAL, text);
}
