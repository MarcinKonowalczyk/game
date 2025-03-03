# As-of-yet untitled game thingy

![screenshot](img/screenshot.png)

A small game project to learn about rust, wasm, javascript and raylib all at the same time ✌️

The project uses [raylib-wasm](https://github.com/rakivo/raylib-wasm) library to link to raylib and compile to wasm. The library includes some functionality, and any other extensions thereto are added in `game/webhacks.rs`, in a similar style.

Assets are not included in the repo to be nice to git in terms of large files. Will become available to download somewhere at some point.

# The tricky bit(s)

Description of some of the trickier buts of this codebase. This section seems to slowly be turing into this project's devlog, and I'm ok with it.

## 🔪 js promisses in rust

The overall idea is that anything from raylib we want to work on the web we need to implement as an ffi on the javascript side. Sometimes this is rather annoying since js is running single-threaded and it *reeeally* wants you to use promises, but on the other hand raylib/rust/wasm (raystsm? rrasm? 🤨) wants things to be blocking. For example `LoadTexture` wants to return a buffer of stuff, including loaded textures' width and height, but by the time hte funciton returns the only thing we can really do synchronously is to generate the id and start the load. Only in `onload` do we know how big the texture is, but by then we already needed to have returned from the function and its too late. Any attempt to wait for the load on either the js or rrasm side results in a nasty page hardlock. Any complex objects we want to return from js to rrasm must therefore be just ids, and any properties must be queried, hence `get_texture_width` and `get_texture_height` (we can get a bunch of properties at once, as long as we don't need to wait for any of them. earlier implementation was just hacked together quickly hence two separate functions instead of `get_texture_size`). This makes the rrasm side a bit less pure-raylib and makes it work a bit harder to try to account for these quirks.

Another upshot of this is that we are not actually done with the init at the end of `game_init`! We've requested all the assets, but they will come at some point in the future, while we have to carry on with the game loop somehow. Hence we set a flag in the state to note whether the loading is actually done or not, and run `game_load` which is like the other half of `game_init` jsut int the loop, or, once loaded, `game_frame` which is the actual game logic.

## 🔪 Serialising state

Any array which is in the `State` obj must be simply serializable (not Vec's for eg, not in the web version anyway). State must, also, have a stable (aka not dynamic) and predictable layout to be able to read it back from js. Hence any dynamically-sized array (e.g. parsed animation frames data) are passed as size and pointer pair.


## 🔪 va_list and over-the-top logging system

C has a concept of a [va_list](https://en.cppreference.com/w/c/variadic/va_list) which is a very hacky, retrofitted and error-prone way of getting variadic functions to work. To get the ryalib logging system to work with our logging function we call `raylib::SetTraceLogCallback` with a pointer to `unsafe extern "C" fn(i32, *const i8, *mut VaList)`. Now, we could format the message ourselves and then call `raylib::TraceLog(level, msg)`. Hooray! Wait.. the messages from raylib internals are messed up... Yeah, here is where that pesky VaList comes into play. The way raylib logging works is that it allows you to call TraceLog with
any number of arguments and it will format it for you. [Rust's VaList](https://doc.rust-lang.org/std/ffi/struct.VaList.html) is currently in unstable and I don't fancy the `+nightly`. Instead, there is a tiny `libvsprintf` which exposes `vsprintf` to rust as an ffi - all of the va_args processign happens on the C-side. `libvsprintf` gets compiled by rust when building game `game` so the entire process is seamless (see the [`cc` crate](https://docs.rs/cc/latest/cc/) ❤️).

So yeah, now we have the logging working, but how about on the js side. Well, we don't need the `vsprintf` (the js effectively *is* the reimplemented raylib backend) but we also want colored output. `console.log("%c hello", "color: green;")` seems to be the only way (as in, you *need* multiple args), so might as well tackle the va_args-like problem but just from rust to js. TLDR; i did not do this *in-general*, but it works for strings. From some reason `*const *const u8` is not our friend in this case (i gave up on trying to figure out why) and so we just join all the strings with null-terminators, add a sentinel value to the end and unpack it on js side.

Another "fun" but is when we want to log "properly" from js. We now have a callback on the rasm side which we need to call. This callback (which, btw, is exported by function *name* not by pointer. another saga... 😮‍💨) takes a text pointer so now we need to alloc it into wasm memory! Luckily by now we actually already have malloc and free exported from rasm, so we just call that and voila! `RuntimeError: Unreachable code should not be executed (evaluating 'WF[LOG_CALLBACK](level, text_ptr)')` 😀... Turns out you have to care about utf8 here. When you make a rust `CStr` from a pointer with `unsafe { std::ffi::CStr::from_ptr(text_ptr) }`, you have to make sure the underlying sequence is a valid utf8, so you can't just merely copy the string char by char into the buffer on js side, but you ned a `TextEncoder` and, of course that pesky null terminator, since now we're traveling to rust side, but rust thinks we're coming from C.

# Quick start

> To run natively:
```sh
cargo run --features=native
```

> To run in browser:
```sh
cargo build --target wasm32-unknown-unknown --features=web && python -m http.server   
<browser> http://0.0.0.0:8000/
```

# Working bits

By 'working' I mean on web and on my machine (macos).

- [x] Font
- [x] Music
- [x] Basic Texture loading
- [x] Basic Animations
- [x] Entities
- [ ] Entity interactions
- [ ] Life/Currency system
- [ ] draw buffer