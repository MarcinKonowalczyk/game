// We do actually want the game since otherwise all of its exports dont appear in the compiled wasm
#[allow(unused)]
#[cfg(feature = "web")]
use game;

#[cfg(feature = "native")]
mod native_runner {
    use libloading::{Library, Symbol};

    use raylib_wasm::{IsKeyPressed, KeyboardKey as Key, WindowShouldClose};

    use game::{GameFrame, GameInit, GameLoad};

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

    const GAME_PATH: &str = get_game_path();

    #[inline]
    fn load_lib(file_path: &str) -> Library {
        unsafe { Library::new(file_path).expect("failed to load the library") }
    }

    #[inline]
    fn load_fn<'lib, T>(lib: &'lib Library, symbol: &str) -> Symbol<'lib, T> {
        unsafe { lib.get(symbol.as_bytes()) }
            .map_err(|err| {
                eprintln!("{err}");
                err
            })
            .unwrap()
    }

    pub fn run() {
        // unsafe {
        //     raylib_wasm::SetTraceLogCallback(Some(my_callback));
        // }
        // log::set_trace_log_callback();
        // log::set_log_level(log::ALL);

        let mut lib = load_lib(GAME_PATH);

        let mut game_frame = load_fn::<Symbol<GameFrame>>(&lib, "game_frame");
        let mut game_load = load_fn::<Symbol<GameLoad>>(&lib, "game_load");
        let game_init = load_fn::<Symbol<GameInit>>(&lib, "game_init");

        let mut state = game_init();

        // log::user("Starting game loop");

        // log::user("Press 'P' to hot-reload the game");
        while !unsafe { WindowShouldClose() } {
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
}

fn main() {
    #[cfg(feature = "native")]
    native_runner::run();
}
