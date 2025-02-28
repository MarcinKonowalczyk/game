#[cfg(feature = "native")]
use libloading::{Library, Symbol};

use raylib_wasm::*;

#[cfg(feature = "native")]
use raylib_wasm::KeyboardKey as Key;

use game::*;

#[cfg(feature = "native")]
const fn get_game_path() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        if cfg!(debug_assertions) {
            concat!("./target/debug/deps/libgame.so")
        } else {
            concat!("./target/release/deps/libgame.so")
        }
    }
    #[cfg(target_os = "windows")]
    {
        if cfg!(debug_assertions) {
            ".\\target\\debug\\deps\\libgame.dll"
        } else {
            ".\\target\\release\\deps\\libgame.dll"
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        if cfg!(debug_assertions) {
            "./target/debug/deps/libgame.dylib"
        } else {
            "./target/release/deps/libgame.dylib"
        }
    }
}

#[cfg(feature = "native")]
const GAME_PATH: &str = get_game_path();

#[inline]
#[cfg(feature = "native")]
fn load_lib(file_path: &str) -> Library {
    unsafe { Library::new(file_path).expect("failed to load the library") }
}

#[inline]
#[cfg(feature = "native")]
fn load_fn<'lib, T>(lib: &'lib Library, symbol: &str) -> Symbol<'lib, T> {
    unsafe { lib.get(symbol.as_bytes()) }
        .map_err(|err| {
            eprintln!("{err}");
            err
        })
        .unwrap()
}

#[cfg(feature = "native")]
mod log {
    use raylib_wasm::*;

    pub const ALL: i32 = 0;
    pub const TRACE: i32 = 1;
    pub const DEBUG: i32 = 2;
    pub const INFO: i32 = 3;
    pub const WARNING: i32 = 4;
    pub const ERROR: i32 = 5;
    pub const FATAL: i32 = 6;
    pub const USER: i32 = 8; // custom user log level
    pub const NONE: i32 = 999;

    const GREEN: &str = "\x1b[92m";
    const YELLOW: &str = "\x1b[93m";
    const RED: &str = "\x1b[91m";
    const BLUE: &str = "\x1b[94m";
    const CYAN: &str = "\x1b[96m";
    const MAGENTA: &str = "\x1b[95m";
    const RESET: &str = "\x1b[0m";

    #[no_mangle]
    pub unsafe extern "C" fn my_log_callback(
        log_level: i32,
        text: *const i8,
        _args: *mut __va_list_tag,
    ) {
        let text = unsafe { std::ffi::CStr::from_ptr(text) };
        let text = text.to_str().unwrap();

        let text = match log_level {
            USER => format!("[{}USER{}] : {}", BLUE, RESET, text),
            INFO => format!("[{}INFO{}] : {}", GREEN, RESET, text),
            ERROR => format!("[{}ERROR{}] : {}", RED, RESET, text),
            WARNING => format!("[{}WARN{}] : {}", YELLOW, RESET, text),
            DEBUG => format!("[{}DEBUG{}] : {}", CYAN, RESET, text),
            TRACE => format!("[{}TRACE{}] : {}", MAGENTA, RESET, text),
            _ => format!("{}", text),
        };

        println!("{}", text);
    }
}

fn start() {
    #[cfg(feature = "native")]
    unsafe {
        raylib_wasm::SetTraceLogCallback(Some(log::my_log_callback));
        raylib_wasm::SetTraceLogLevel(log::ALL);
    }

    #[cfg(feature = "native")]
    let mut lib = load_lib(GAME_PATH);

    #[cfg(feature = "native")]
    let mut game_frame = load_fn::<Symbol<GameFrame>>(&lib, "game_frame");

    #[cfg(feature = "native")]
    let mut game_load = load_fn::<Symbol<GameLoad>>(&lib, "game_load");

    let mut state = game_init();

    // println!("Starting game loop");
    // println!("Press 'P' to hot-reload the game");
    #[cfg(feature = "native")]
    unsafe {
        raylib_wasm::TraceLog(log::USER, cstr!("Starting game loop"));
        raylib_wasm::TraceLog(log::USER, cstr!("Press 'P' to hot-reload the game"));
    }

    while !unsafe { WindowShouldClose() } {
        #[cfg(feature = "native")]
        if unsafe { IsKeyPressed(Key::P) } {
            drop(game_frame);
            drop(game_load);
            drop(lib);
            lib = load_lib(GAME_PATH);
            game_frame = load_fn(&lib, "game_frame");
            game_load = load_fn(&lib, "game_load");
        }

        if state.all_loaded.bool() {
            game_frame(&mut state);
        } else {
            game_load(&mut state);
        }
    }
}

fn main() {
    start();
}
