'use strict';

const WASM_PATH = "./target/wasm32-unknown-unknown/debug/hotreload-raylib-wasm-template.wasm"
const FONT_SCALE_MAGIC = 0.75;

// const MOUSE_MAP = {
//     0: "Left",
//     1: "Right",
//     2: "Middle",
//     3: "Side",
//     4: "Extra",
//     5: "Forward",
//     6: "Back",
// }

const GLFW_MAP = {
    "Space": 32,
    "Quote": 39,
    "Comma": 44,
    "Minus": 45,
    "Period": 46,
    "Slash": 47,
    "Digit0": 48,
    "Digit1": 49,
    "Digit2": 50,
    "Digit3": 51,
    "Digit4": 52,
    "Digit5": 53,
    "Digit6": 54,
    "Digit7": 55,
    "Digit8": 56,
    "Digit9": 57,
    "Semicolon": 59,
    "Equal": 61,
    "KeyA": 65,
    "KeyB": 66,
    "KeyC": 67,
    "KeyD": 68,
    "KeyE": 69,
    "KeyF": 70,
    "KeyG": 71,
    "KeyH": 72,
    "KeyI": 73,
    "KeyJ": 74,
    "KeyK": 75,
    "KeyL": 76,
    "KeyM": 77,
    "KeyN": 78,
    "KeyO": 79,
    "KeyP": 80,
    "KeyQ": 81,
    "KeyR": 82,
    "KeyS": 83,
    "KeyT": 84,
    "KeyU": 85,
    "KeyV": 86,
    "KeyW": 87,
    "KeyX": 88,
    "KeyY": 89,
    "KeyZ": 90,
    "BracketLeft": 91,
    "Backslash": 92,
    "BracketRight": 93,
    "Backquote": 96,
    "Escape": 256,
    "Enter": 257,
    "Tab": 258,
    "Backspace": 259,
    "Insert": 260,
    "Delete": 261,
    "ArrowRight": 262,
    "ArrowLeft": 263,
    "ArrowDown": 264,
    "ArrowUp": 265,
    "PageUp": 266,
    "PageDown": 267,
    "Home": 268,
    "End": 269,
    "CapsLock": 280,
    "ScrollLock": 281,
    "NumLock": 282,
    "PrintScreen": 283,
    "Pause": 284,
    "F1": 290,
    "F2": 291,
    "F3": 292,
    "F4": 293,
    "F5": 294,
    "F6": 295,
    "F7": 296,
    "F8": 297,
    "F9": 298,
    "F10": 299,
    "F11": 300,
    "F12": 301,
    "F13": 302,
    "F14": 303,
    "F15": 304,
    "F16": 305,
    "F17": 306,
    "F18": 307,
    "F19": 308,
    "F20": 309,
    "F21": 310,
    "F22": 311,
    "F23": 312,
    "F24": 313,
    "F25": 314,
    "NumPad0": 320,
    "NumPad1": 321,
    "NumPad2": 322,
    "NumPad3": 323,
    "NumPad4": 324,
    "NumPad5": 325,
    "NumPad6": 326,
    "NumPad7": 327,
    "NumPad8": 328,
    "NumPad9": 329,
    "NumpadDecimal": 330,
    "NumpadDivide": 331,
    "NumpadMultiply": 332,
    "NumpadSubtract": 333,
    "NumpadAdd": 334,
    "NumpadEnter": 335,
    "NumpadEqual": 336,
    "ShiftLeft": 340,
    "ControlLeft": 341,
    "AltLeft": 342,
    "MetaLeft": 343,
    "ShiftRight": 344,
    "ControlRight": 345,
    "AltRight": 346,
    "MetaRight": 347,
    "ContextMenu": 348,
}

function cstrlen(mem, ptr) {
    let len = 0;
    while (mem[ptr] != 0) {
        len++;
        ptr++;
    }
    return len;
}

function cstr_by_ptr(mem_buffer, ptr) {
    const mem = new Uint8Array(mem_buffer);
    const len = cstrlen(mem, ptr);
    const bytes = new Uint8Array(mem_buffer, ptr, len);
    return new TextDecoder().decode(bytes);
}

// pub struct Color {
//     pub r: u8,
//     pub g: u8,
//     pub b: u8,
//     pub a: u8,
// }
function getColorFromMemory(buffer, color_ptr) {
    var [r, g, b, a] = new Uint8Array(buffer, color_ptr, 4);
    r = r.toString(16).padStart(2, '0');
    g = g.toString(16).padStart(2, '0');
    b = b.toString(16).padStart(2, '0');
    a = a.toString(16).padStart(2, '0');
    return "#" + r + g + b + a;
}

// pub struct Rectangle {
//     pub x: f32,
//     pub y: f32,
//     pub width: f32,
//     pub height: f32,
// }
function getRectangleFromMemory(buffer, rec_ptr) {
    let mem = new Float32Array(buffer, rec_ptr, 4);
    return { x: mem[0], y: mem[1], width: mem[2], height: mem[3] };
}
// pub struct Vector2 {
//     pub x: f32,
//     pub y: f32,
// }
function getVector2FromMemory(buffer, vec_ptr) {
    let mem = new Float32Array(buffer, vec_ptr, 2);
    return { x: mem[0], y: mem[1] };
}

function make_environment(...envs) {
    return new Proxy(envs, {
        get(target, prop, receiver) {
            for (let env of envs) if (env.hasOwnProperty(prop)) return env[prop];
            return (...args) => { console.error("NOT IMPLEMENTED: " + prop, args) }
        }
    });
}

let prev_pressed_key = new Set();
let curr_pressed_key = new Set();

const keyDown = (e) => {
    e.preventDefault();
    curr_pressed_key.add(GLFW_MAP[e.code]);
}

const keyUp = (e) => {
    e.preventDefault();
    curr_pressed_key.delete(GLFW_MAP[e.code]);
}

const game = document.getElementById("game");
var container = game.parentElement; // parent div
const ctx = game.getContext("2d");

game.mouseX = -1;
game.mouseY = -1;
game.mouseDown = false;
game.mouseButton = -1;

game.onmousemove = handleMouseMove;

function handleMouseMove(event) {
    var rect = container.getBoundingClientRect();
    var xf = event.offsetX / rect.width;
    var yf = event.offsetY / rect.height;
    game.mouseX = xf * game.width;
    game.mouseY = yf * game.height;
}

game.onmouseleave = function (event) {
    // console.log("mouse leave");
    game.mouseX = -1;
    game.mouseY = -1;
}

game.onmousedown = function (event) {
    // console.log("mouse down");
    game.mouseDown = true;
    game.mouseButton = event.button;
}

game.onmouseup = function (event) {
    // console.log("mouse up");
    game.mouseDown = false;
    game.mouseButton = -1;
}

game.oncontextmenu = function (event) {
    // console.log("right click");
    event.preventDefault();
}

// game.onmouseenter = function (event) {
//     console.log("mouse enter");
// }

var SCALE_TO_FIT = true;
var WIDTH = 800;
var HEIGHT = 600;

function onResize() {
    var w;
    var h;

    if (SCALE_TO_FIT) {
        w = window.innerWidth;
        h = window.innerHeight;

        var r = HEIGHT / WIDTH;

        if (w * r > window.innerHeight) {
            w = Math.min(w, Math.ceil(h / r));
        }
        h = Math.floor(w * r);
    } else {
        w = WIDTH;
        h = HEIGHT;
    }

    container.style.width = game.style.width = w + "px";
    container.style.height = game.style.height = h + "px";
    container.style.top = Math.floor((window.innerHeight - h) / 2) + "px";
    container.style.left = Math.floor((window.innerWidth - w) / 2) + "px";
}
window.addEventListener('resize', onResize);

onResize();

if (/iPhone|iPad|iPod|Android/i.test(navigator.userAgent)) {
    // Mobile device style: fill the whole browser client area with the game canvas:
    const meta = document.createElement('meta');
    meta.name = 'viewport';
    meta.content = 'width=device-width, height=device-height, initial-scale=1.0, user-scalable=no, shrink-to-fit=yes';
    document.getElementsByTagName('head')[0].appendChild(meta);
}

let audio = {
    loop: undefined,
}

function initAudioContext(url) {
    loopify(url, function (err, loop) {
        // If something went wrong, `err` is supplied
        if (err) {
            return console.err(err);
        }
        audio.loop = loop;
    });
}

function tryToPlayAudio() {
    if (audio.loop === undefined) {
        // no audio loaded
        return;
    }
    if (audio.loop.playing()) {
        return;
    }
    audio.loop.play(0.0);
}

let images = new Map();
let textures = new Map();
let wasm = undefined;
let dt = undefined;
let wf = undefined;
let quit = undefined;
let prev = undefined;
let targetFPS = undefined;
let font_map = new Map();

const GetFPS = () => 1.0 / dt;

WebAssembly.instantiateStreaming(fetch(WASM_PATH), {
    "env": make_environment({
        ConsoleLog_ (text_ptr) {
            const buffer = wf.memory.buffer;
            const text = cstr_by_ptr(buffer, text_ptr);
            console.log(text);
        },
        GetMousePositionX: () => game.mouseX,
        GetMousePositionY: () => game.mouseY,
        IsMouseButtonDown: (button) => {
            // console.log(button, game.mouseButton);
            return game.mouseButton === button;
        },
        InitWindow: (w, h, t) => {
            game.width = w;
            game.height = h;
            const buffer = wf.memory.buffer;
            document.title = cstr_by_ptr(buffer, t);
        },
        BeginDrawing: () => { },
        SetExitKey: () => { },
        CloseWindow: () => { },
        EndDrawing: () => {
            prev_pressed_key.clear();
            prev_pressed_key = new Set(curr_pressed_key);
        },
        IsKeyReleased: (key) => prev_pressed_key.has(key) && !curr_pressed_key.has(key),
        IsKeyDown: (key) => curr_pressed_key.has(key),
        ClearBackground: (color_ptr) => {
            const buffer = wf.memory.buffer;
            const color = getColorFromMemory(buffer, color_ptr);
            ctx.fillStyle = color;
            ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
        },
        MeasureText: (text_ptr, fontSize) => {
            const buffer = wasm.instance.exports.memory.buffer;
            const text = cstr_by_ptr(buffer, text_ptr);
            fontSize *= FONT_SCALE_MAGIC;
            ctx.font = `${fontSize}px grixel`;
            return ctx.measureText(text).width;
        },
        DrawText: (text_ptr, posX, posY, fontSize, color_ptr) => {
            const buffer = wf.memory.buffer;
            const text = cstr_by_ptr(buffer, text_ptr);
            const color = getColorFromMemory(buffer, color_ptr);
            fontSize *= FONT_SCALE_MAGIC;
            ctx.fillStyle = color;
            ctx.font = `${fontSize}px grixel`;
            const lines = text.split('\n');
            for (var i = 0; i < lines.length; i++) {
                ctx.fillText(lines[i], posX, posY + fontSize + (i * fontSize));
            }
        },
        LoadFont: (file_path_ptr) => {
            const buffer = wf.memory.buffer;
            const file_path = cstr_by_ptr(buffer, file_path_ptr);

            // split at the last slash and at the last dot
            let ext = file_path.split('.').pop();
            let font_name = file_path.split('/').pop().split('.').slice(0, -1).join('.');

            if (font_map.has(font_name)) {
                // font already loaded
                return font_map.get(font_name);
            }

            // generate a unique id for the font
            var font_id = Math.floor(Math.random() * 1000000);
            
            // fetch the font file
            fetch(file_path).then((response) => {
                return response.arrayBuffer();
            }).then((buffer) => {
                return new Promise((resolve, reject) => {
                    const reader = new FileReader();
                    reader.onload = () => resolve(reader.result);
                    reader.onerror = reject;
                    reader.readAsArrayBuffer(new Blob([buffer]));
                });
            }).then((buffer) => {
                return new Promise((resolve, reject) => {
                    const font = new FontFace(font_name, buffer);
                    font.load().then((loaded_face) => {
                        document.fonts.add(loaded_face);
                        resolve(font);
                    }).catch(reject);
                });
            }).then((font) => {
                font_map.set(font_id, font_name);
                return font_id;
            }).catch((err) => {
                console.log(err);
                return -1;
            });

            return font_id;
        },
        IsFontLoaded: (font) => {
            return font_map.has(font);
        },
        DrawTextEx_: (font, text_ptr, posX, posY, fontSize, spacing,  color_ptr) => {
            const buffer = wf.memory.buffer;
            const text = cstr_by_ptr(buffer, text_ptr);
            const color = getColorFromMemory(buffer, color_ptr);
            fontSize *= FONT_SCALE_MAGIC;
            ctx.fillStyle = color;

            var font_name = font_map.get(font);
            if (font_name === undefined) {
                console.log("Font not found", font_map, font);
                return;
            }

            ctx.font = `${fontSize}px ${font_name}`;
            
            const lines = text.split('\n');
        
            for (var i = 0; i < lines.length; i++) {
                const chars = lines[i].split('');
                let x = posX;
                for (var j = 0; j < chars.length; j++) {
                    ctx.fillText(chars[j], x, posY + fontSize + (i * fontSize));
                    x += ctx.measureText(chars[j]).width + spacing;
                }
                // ctx.fillText(lines[i], posX, posY + fontSize + (i * fontSize));
            }
        },
        DrawLine: (startPosX, startPosY, endPosX, endPosY, color_ptr) => {
            const buffer = wf.memory.buffer;
            const color = getColorFromMemory(buffer, color_ptr);
            ctx.fillStyle = color;
            ctx.beginPath();
            ctx.moveTo(startPosX, startPosY);
            ctx.lineTo(endPosX, endPosY);
            ctx.strokeStyle = color;
            ctx.stroke();
        },
        DrawRectangle: (posX, posY, width, height, color_ptr) => {
            const buffer = wf.memory.buffer;
            const color = getColorFromMemory(buffer, color_ptr);
            ctx.fillStyle = color;
            ctx.fillRect(posX, posY, width, height);
        },
        DrawRectangleV: (position_ptr, size_ptr, color_ptr) => {
            const buffer = wf.memory.buffer;
            const [x, y] = new Float32Array(buffer, position_ptr, 2);
            const [width, height] = new Float32Array(buffer, size_ptr, 2);
            const color = getColorFromMemory(buffer, color_ptr);
            ctx.fillStyle = color;
            ctx.fillRect(x, y, width, height);
        },
        DrawRectangleRec: (rec_ptr, color_ptr) => {
            const buffer = wf.memory.buffer;
            const [x, y, w, h] = new Float32Array(buffer, rec_ptr, 4);
            const color = getColorFromMemory(buffer, color_ptr);
            ctx.fillStyle = color;
            ctx.fillRect(x, y, w, h);
        },
        DrawCircle: (centerX, centerY, radius, color_ptr) => {
            const buffer = wf.memory.buffer;
            const color = getColorFromMemory(buffer, color_ptr);
            ctx.fillStyle = color;
            ctx.beginPath();
            ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI, 0);
            ctx.fill();
        },
        LoadTexture: (file_path_ptr) => {
            console.log("Loading texture");
            // console.log(result_ptr, file_path_ptr);
            const buffer = wf.memory.buffer;
            console.log("1");
            const file_path = cstr_by_ptr(buffer, file_path_ptr);
            console.log("2");

            // let result = new Uint32Array(buffer, result_ptr, 5)
            let img = new Image();
            var id = Math.floor(Math.random() * 1000000);

            console.log(id);

            textures[id] = img;

            // Some info we already know
            // result[0] = id;
            // result[3] = 1; // mipmaps
            // result[4] = 7; // format PIXELFORMAT_UNCOMPRESSED_R8G8B8A8

            console.log("Loading image", id, file_path);
            // img.onload = () => {
            //     textures[id] = img;
            // };
            img.src = file_path;
            
            // console.log(result);
            
            return id;
        },
        IsTextureLoaded: (id) => {
            return textures[id].complete;
        },
        GetTextureWidth: (id) => {
            const img = textures[id];
            if (img === undefined) {
                return 0;
            }
            return img.width;
        },
        GetTextureHeight: (id) => {
            const img = textures[id];
            if (img === undefined) {
                return 0;
            }
            return img.height;
        },
        // pub fn DrawTextureEx_(
        //     texture: Texture,
        //     positionX: i32,
        //     positionY: i32,
        //     rotation: f32,
        //     scale: f32,
        //     tint: *const Color,
        // );
        DrawTextureEx_: (id, x, y, rotation, scale, _color_ptr) => {
            const img = textures[id];
            ctx.save();
            ctx.translate(x, y);
            ctx.rotate(rotation);
            ctx.scale(scale, scale);
            ctx.drawImage(img, 0, 0);
            ctx.restore();
        },
        // pub fn DrawTexturePro_(
        //     texture: Texture,
        //     sourceRec: raylib::Rectangle,
        //     destRec: raylib::Rectangle,
        //     origin: raylib::Vector2,
        //     rotation: f32,
        //     tint: *const Color,
        // );
        DrawTexturePro_: (id, sourceRec_ptr, destRec_ptr) => {
            const img = textures[id];
            const buffer = wf.memory.buffer;
            const sourceRec = getRectangleFromMemory(buffer, sourceRec_ptr);
            const destRec = getRectangleFromMemory(buffer, destRec_ptr);
            ctx.save();
            ctx.translate(destRec.x, destRec.y);
            ctx.drawImage(img, sourceRec.x, sourceRec.y, sourceRec.width, sourceRec.height, 0, 0, destRec.width, destRec.height);
            ctx.restore();
        },
        UnloadTexture: () => { },
        GetScreenWidth: () => ctx.canvas.width,
        GetScreenHeight: () => ctx.canvas.height,
        GetFrameTime: () => {
            if (targetFPS !== undefined) {
                return Math.min(dt, 1.0 / targetFPS);
            }
            return dt;
        },
        IsWindowResized: () => false,
        WindowShouldClose: () => false,
        SetTargetFPS: (x) => targetFPS = x,
        GetFPS: () => GetFPS(),
        // DrawFPS: (x, y) => {
        //     const fontSize = 50.0 * FONT_SCALE_MAGIC;
        //     const fps = GetFPS();
        //     let color = "lime";                               // Good FPS
        //     if ((fps < 30) && (fps >= 15)) color = "orange";  // Warning FPS
        //     else if (fps < 15) color = "red";                 // Low FPS
        //     ctx.fillStyle = "green";
        //     ctx.font = `${fontSize}px grixel`;
        //     ctx.fillText(targetFPS, x, y + fontSize);
        // },
        alert: (ptr) => {
            let msg = cstr_by_ptr(ptr);
            console.log(msg);
            window.alert(msg);
        },
        InitAudioDevice: () => { },
        LoadMusicStream: (file_path_ptr) => {
            const buffer = wf.memory.buffer;
            const file_path = cstr_by_ptr(buffer, file_path_ptr);

            let audio_id = Math.floor(Math.random() * 1000000);
            console.log("Loading music stream", audio_id, file_path);

            // Wait for the file fo be fetched
            fetch(file_path).then((response) => {
                console.log(response);
                initAudioContext(response.url);
            }).catch((err) => {
                console.log(err);
            });

            return audio_id;
        },
        IsMusicLoaded: () => {
            return audio.loop !== undefined;
        },
        PlayMusicStream: (_audio_id) => {
            tryToPlayAudio();
        },
        UpdateMusicStream: (_audio_id) => {
            tryToPlayAudio();
        },
        // pub fn LoadImage(file_path: *const i8) -> u32;
        LoadImage: (file_path_ptr) => {
            const buffer = wf.memory.buffer;
            const file_path = cstr_by_ptr(buffer, file_path_ptr);
            let img = new Image();
            var id = Math.floor(Math.random() * 1000000);

            images[id] = img;

            img.src = file_path;

            img.onload = () => {
                console.log("Image loaded", id);
            }

            return id;
        },
        // pub fn LoadImageColors(image: Image) -> *mut Color;
        // pub struct Color {
        //     pub r: u8,
        //     pub g: u8,
        //     pub b: u8,
        //     pub a: u8,
        // }
        LoadImageColors: (image_id) => {
            // colors are an array of Color
            const img = images[image_id];
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);
            const data = ctx.getImageData(0, 0, img.width, img.height).data;
            console.log("Image data", data);
            const colors = new Uint8Array(wf.memory.buffer, wf.from_js_malloc(data.length), data.length);
            colors.set(data);
            console.log("Image colors", colors);
            return colors.byteOffset;
        },
        UnloadImageColors: (colors_ptr, size) => {
            wf.from_js_free(colors_ptr, size);
        },
        IsImageLoaded: (image_id) => {
            return images[image_id].complete;
        },
        // pub fn LoadTextureFromImage(image: u32) -> u32;
        LoadTextureFromImage: (image_id) => {
            const img = images[image_id];
            var tex_id = Math.floor(Math.random() * 1000000);
            console.log("Loading texture from image. Image id: %d, Texture id: %d", image_id, tex_id);
            textures[tex_id] = img;
            return tex_id;
        },
        // pub fn GetImageWidth(image: u32) -> i32;
        GetImageWidth: (image_id) => {
            const img = images[image_id];
            if (img === undefined) {
                return 0;
            }
            return img.width;
        },
        // pub fn GetImageHeight(image: u32) -> i32;
        GetImageHeight: (image_id) => {
            const img = images[image_id];
            if (img === undefined) {
                return 0;
            }
            return img.height;
        },
        // pub fn UnloadImage(image: Image) -> ();
        UnloadImage: (image_id) => {
            delete images[image_id];
        },
        // pub fn GetTime() -> f64;
        GetTime: () => {
            let t = performance.now();
            return t / 1000;
        }
    })
}).then(w => {
    wasm = w;
    wf = w.instance.exports;
    console.log(w);

    window.addEventListener("keydown", keyDown);
    window.addEventListener("keyup", keyUp);

    let ptr = wf.test();

    function read_my_state(ptr) {
        const buffer = wasm.instance.exports.memory.buffer;

        var data_view = new DataView(buffer, ptr);
        var i = 0;
        var a = data_view.getUint8(i); i += 1;
        var b = data_view.getFloat32(i, true); i += 4;
        var c = data_view.getFloat32(i, true); i += 4;
        var d = data_view.getFloat32(i, true); i += 4;
        var e = data_view.getFloat32(i, true); i += 4;
        
        var N_C = data_view.getUint32(i, true); i += 4;
        var ptr_C = data_view.getUint32(i, true); i += 4;

        var C = {};
        for (i = 0; i < N_C; i++) {
            var data_view = new DataView(buffer, ptr_C + i * 16, 16);
            var d1 = data_view.getFloat32(0, true);
            var d2 = data_view.getFloat32(4, true);
            var d3 = data_view.getFloat32(8, true);
            var d4 = data_view.getFloat32(12, true);
            C[i] = { d1, d2, d3, d4 };
        }

        console.log(C);

        return { a, b, c, d, e };
    }

    let my_state = read_my_state(ptr);

    console.log(my_state);

    let state = wf.game_init();

    function read_loaded_flag(ptr) {
        const buffer = wasm.instance.exports.memory.buffer;
        var data_view = new DataView(buffer, ptr, 4);
        return data_view.getUint32(0, true) == 1;
    }

    function read_state(ptr) {
        const buffer = wasm.instance.exports.memory.buffer;

        // state buffer is 4-byte aligned.
        var data_view = new DataView(buffer, ptr, 256);

        let schema = "bu[ffff]f{speed}[ff]bu{music}u{font}u{texture}[u{x_max}uuu]*";

        function data_view_to_struct(data_view, schema) {

            let tokens = [];
            for (let token of scanner(schema)) {
                tokens.push(token);
            }

            console.log("tokens:", tokens);

            function _data_view_to_struct(data_view, tokens, offset) {

                if (offset === undefined) {
                    offset = 0;
                }

                var out = [];
                let i = 0;
                for (let token of tokens) {

                    if (token.type === "error") {
                        console.error("Error parsing schema", token);
                        return;
                    } 

                    if (token.is_array) {
                        if (token.type === "struct") {
                            // parse array of structs
                            // first read the length of the array
                            console.log("parsing array of structs");
                            let len = data_view.getUint32(i + offset, true);
                            i += 4;
                            console.log("array length", len);
                            let arr = [];
                            for (let j = 0; j < len; j++) {
                                let s = _data_view_to_struct(data_view, token.value, i);
                                arr.push(s[0]);
                                i += s[1];
                            }
                            console.log("parsed array of structs", arr);
                            out.push(arr);
                        }
                    } else {
                        // parse single token
                        if (token.type === "uint32") {
                            out.push(data_view.getUint32(i + offset, true));
                            i += 4;
                        } else if (token.type === "float32") {
                            out.push(data_view.getFloat32(i + offset, true));
                            i += 4;
                        } else if (token.type === "bool") {
                            // We are 4-byte aligned, so a bool takes 4 bytes
                            let temp = data_view.getUint32(i + offset);
                            console.log(temp);
                            out.push(data_view.getUint32(i + offset) === 1);
                            i += 4;
                        } else if (token.type === "struct") {
                            // recursively parse the struct
                            let s = _data_view_to_struct(data_view, token.value, i);
                            out.push(s[0]);
                            i += s[1];
                        } else {
                            console.error("Unknown token type", token);
                        }
                    }
                }

                return [ out, i ];
    
            }
            
            let out = {};
            let i = 0;
            [out, i] = _data_view_to_struct(data_view, tokens);

            console.log(out, i);


        }
        
        data_view_to_struct(data_view, schema);

    }

    let parsed_state = read_state(ptr);

    console.log(parsed_state);

    const next = (timestamp) => {
        if (quit) {
            ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
            window.removeEventListener("keydown", keyDown);
            return;
        }
        dt = (timestamp - prev) / 1000.0;
        prev = timestamp;
        
        if (read_loaded_flag(ptr)) {
            wf.game_frame(state);
        } else {
            wf.game_load(state);
        }
        window.requestAnimationFrame(next);
        // DEBUG: slow down the loop
        // setTimeout(() => {
        //     window.requestAnimationFrame(next);
        // }, 5000
        // );
    };
    window.requestAnimationFrame((timestamp) => {
        prev = timestamp;
        window.requestAnimationFrame(next);
    });
}).catch((err) => {
    console.log(err);
    console.log('update WASM_PATH in `main.js` bruv!');
});


///////////////////////////////////////////

// loopify.js
// https://github.com/veltman/loopify
// v0.1-modified

// Available under the MIT license.

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions.
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


function loopify(uri, cb) {

    var context = new (window.AudioContext || window.webkitAudioContext)();
    var request = new XMLHttpRequest();

    var obj = undefined;

    // If we have not interacted with the page, we can't play audio
    // Try to resume it every 100ms, only once successful we can play
    var can_play = false;
    var want_to_play = false; // if we want to play but can't yet
    var resume_timeout = 100;

    const timeout = (prom, time) => {
        return Promise.race([prom, new Promise((_r, rej) => setTimeout(rej, time))])
    };

    function resume() {
        timeout(context.resume(), resume_timeout).then(() => {

            // Context is resumed! We can play audio now.
            can_play = true;

            // I we want to play, do it now
            if (want_to_play) {
                want_to_play = false;
                if (obj !== undefined) {
                    obj.play();
                }
            }
        }, resume);
    }

    resume();

    request.responseType = "arraybuffer";
    request.open("GET", uri, true);

    // XHR failed
    request.onerror = function () {
        cb(new Error("Couldn't load audio from " + uri));
    };

    // XHR complete
    request.onload = function () {
        context.decodeAudioData(request.response, success, function (err) {
            // Audio was bad
            cb(new Error("Couldn't decode audio from " + uri));
        });
    };

    request.send();

    function success(buffer) {

        var source;
        var future_id; // id of the timeout for the next play

        function canPlay() {
            return can_play;
        }

        function play(fade_time) {

            if (fade_time === undefined) {
                fade_time = 0.0;
            }

            // We cannot play yet, but maybe this was triggered by our first
            // interaction with the page, and we will be able to play soon.
            // There is a race between call to play and the callback of the
            // resume of the context. We just set a flag here, and return.
            // The resume callback will check this flag and play if needed.
            if (!can_play) {
                // We can't play audio yet
                want_to_play = true;
                return;
            }

            // Stop if it's already playing
            stop();

            // Called at the start of the new segment, 'fade_time' before
            // the end of the previous one
            function playSegment(prev_gain) {
                var now = context.currentTime;

                // Create a new source (can't replay an existing source)
                source = context.createBufferSource();
                var gain = context.createGain();
                source.connect(gain).connect(context.destination);
                source.buffer = buffer;

                // Fade in this segment
                gain.gain.setValueAtTime(0, now);
                gain.gain.linearRampToValueAtTime(1, now + fade_time);

                // Crossfade with previous segment if it exists
                if (prev_gain !== undefined) {
                    prev_gain.gain.linearRampToValueAtTime(0, now + fade_time);
                }

                // start source
                source.start(now);

                return gain;
            }

            // Play segment and recursively schedule the next one
            function recursivePlay(prev_gain) {
                // Play the current segment
                var gain = playSegment(prev_gain);

                // Schedule ourselves to play the next segment
                future_id = setTimeout(() => {
                    recursivePlay(gain);
                }, (buffer.duration - fade_time) * 1000);
            }

            recursivePlay();

        }

        function stop() {

            want_to_play = false;

            // Stop and clear if it's playing
            if (source) {
                source.stop();
                source = null;
            }

            // Clear any future play timeouts
            if (future_id) {
                clearTimeout(future_id);
                future_id = null;
            }

        }

        function playing() {
            return source !== undefined;
        }

        // Return the object to the callback
        obj = {
            play: play,
            stop: stop,
            playing: playing,
        }

        cb(null, obj);

    }

}

loopify.version = "0.2";

///////////////////////////////////////////


// let schema = "u[ffff]f*{speed}[ff]bu{music}u{font}u{texture}[u{x_max}uuu]*";

function parse_until(schema, i, end) {
    let label = "";
    i++;
    while (schema[i] !== end) {
        label += schema[i];
        i++;
    }
    return [ label, i ];
}

function* scanner(schema) {
    let i = 0;
    while (i < schema.length) {
        var out;
        let char = schema[i];
        if (char === "u") {
            out = { type: "uint32", value: char }
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [ out.label, i ] = parse_until(schema, ++i, "}");
            yield out;
        } else if (char === "f") {
            out = { type: "float32", value: char };
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [ out.label, i ] = parse_until(schema, ++i, "}");
            yield out;
        } else if (char === "b") {
            out = { type: "bool", value: char };
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [ out.label, i ] = parse_until(schema, ++i, "}");
            yield out;
        } else if (char === "[") {
            out = { type: "struct"};
            var content = "";
            [ content, i ] = parse_until(schema, i, "]");
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [ out.label, i ] = parse_until(schema, ++i, "}");

            // recursively parse the content
            out.value = [];
            for (let token of scanner(content)) {
                out.value.push(token);
            }
            yield out;
        } else if (char === " " || char === "," || char === "\n") {
            // silently skip whitespace and commas and newlines
            i++;
        } else {
            yield { type: "error", value: char };
        }

        i++;
    }
}