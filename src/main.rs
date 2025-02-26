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

fn start() {
    #[cfg(feature = "native")]
    let mut lib = load_lib(GAME_PATH);

    #[cfg(feature = "native")]
    let mut game_frame = load_fn::<Symbol<GameFrame>>(&lib, "game_frame");

    #[cfg(feature = "native")]
    let mut game_load = load_fn::<Symbol<GameLoad>>(&lib, "game_load");

    let mut state = game_init();
    while !unsafe { WindowShouldClose() } {
        #[cfg(feature = "native")]
        if unsafe { IsKeyPressed(Key::R) } {
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
