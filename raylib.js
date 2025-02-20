'use strict';

import { loopify } from './loopify.js';
import { wasm_to_struct } from './wasm_struct_parser.js';
import { getString, getRectangle, getColor, getVector2 } from './mem_helpers.js';
import { GLFW_MAP } from './glfw_map.js';

const WASM_PATH = "./target/wasm32-unknown-unknown/debug/hotreload-raylib-wasm-template.wasm"
const FONT_SCALE_MAGIC = 0.75;

let ALL_IDS = new Set();

function gen_asset_id() {
    let _gen_id = () => Math.floor(Math.random() * 1000000);
    var id = _gen_id();
    while (ALL_IDS.has(id) || id === 0) {
        id = _gen_id();
    }
    ALL_IDS.add(id);
    return id;
}

function drop_asset_id(id) {
    ALL_IDS.delete(id);
}

function make_environment(...envs) {
    return new Proxy(envs, {
        get(target, prop, receiver) {
            for (let env of envs) if (env.hasOwnProperty(prop)) return env[prop];
            return (...args) => { console.error("NOT IMPLEMENTED: " + prop, args) }
        }
    });
}

let PREV_PRESSED_KEY = new Set();
let CURR_PRESSED_KEY = new Set();

const keyDown = (e) => {
    e.preventDefault();
    CURR_PRESSED_KEY.add(GLFW_MAP[e.code]);
}

const keyUp = (e) => {
    e.preventDefault();
    CURR_PRESSED_KEY.delete(GLFW_MAP[e.code]);
}

const GAME = document.getElementById("game");
var CONTAINER = GAME.parentElement; // parent div
const CTX = GAME.getContext("2d");

GAME.mouseX = -1;
GAME.mouseY = -1;
GAME.mouseDown = false;
GAME.mouseButton = -1;

GAME.onmousemove = handleMouseMove;

function handleMouseMove(event) {
    var rect = CONTAINER.getBoundingClientRect();
    var xf = event.offsetX / rect.width;
    var yf = event.offsetY / rect.height;
    GAME.mouseX = xf * GAME.width;
    GAME.mouseY = yf * GAME.height;
}

GAME.onmouseleave = function (event) {
    // console.log("mouse leave");
    GAME.mouseX = -1;
    GAME.mouseY = -1;
}

GAME.onmousedown = function (event) {
    // console.log("mouse down");
    GAME.mouseDown = true;
    GAME.mouseButton = event.button;
}

GAME.onmouseup = function (event) {
    // console.log("mouse up");
    GAME.mouseDown = false;
    GAME.mouseButton = -1;
}

GAME.oncontextmenu = function (event) {
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

    CONTAINER.style.width = GAME.style.width = w + "px";
    CONTAINER.style.height = GAME.style.height = h + "px";
    CONTAINER.style.top = Math.floor((window.innerHeight - h) / 2) + "px";
    CONTAINER.style.left = Math.floor((window.innerWidth - w) / 2) + "px";
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

let IMAGES = new Map();
let TEXTURES = new Map();
let FONTS = new Map();

let WASM = undefined;
let DT = undefined;
let WF = undefined;
let QUIT = undefined;
let _PREV_TIMESTAMP = undefined;
let TARGET_FPS = undefined;

WebAssembly.instantiateStreaming(fetch(WASM_PATH), {
    "env": make_environment({
        ConsoleLog_(text_ptr) {
            const buffer = WF.memory.buffer;
            const text = getString(buffer, text_ptr);
            console.log(text);
        },
        GetMousePositionX: () => GAME.mouseX,
        GetMousePositionY: () => GAME.mouseY,
        IsMouseButtonDown: (button) => {
            // console.log(button, game.mouseButton);
            return GAME.mouseButton === button;
        },
        InitWindow: (w, h, t) => {
            GAME.width = w;
            GAME.height = h;
            const buffer = WF.memory.buffer;
            document.title = getString(buffer, t);
        },
        BeginDrawing: () => { },
        SetExitKey: () => { },
        CloseWindow: () => { },
        EndDrawing: () => {
            PREV_PRESSED_KEY.clear();
            PREV_PRESSED_KEY = new Set(CURR_PRESSED_KEY);
        },
        IsKeyReleased: (key) => PREV_PRESSED_KEY.has(key) && !CURR_PRESSED_KEY.has(key),
        IsKeyDown: (key) => CURR_PRESSED_KEY.has(key),
        ClearBackground: (color_ptr) => {
            const buffer = WF.memory.buffer;
            const color = getColor(buffer, color_ptr);
            CTX.fillStyle = color;
            CTX.fillRect(0, 0, CTX.canvas.width, CTX.canvas.height);
        },
        MeasureText: (text_ptr, fontSize) => {
            const buffer = WASM.instance.exports.memory.buffer;
            const text = getString(buffer, text_ptr);
            fontSize *= FONT_SCALE_MAGIC;
            CTX.font = `${fontSize}px grixel`;
            return CTX.measureText(text).width;
        },
        DrawText: (text_ptr, posX, posY, fontSize, color_ptr) => {
            const buffer = WF.memory.buffer;
            const text = getString(buffer, text_ptr);
            const color = getColor(buffer, color_ptr);
            fontSize *= FONT_SCALE_MAGIC;
            CTX.fillStyle = color;
            CTX.font = `${fontSize}px grixel`;
            const lines = text.split('\n');
            for (var i = 0; i < lines.length; i++) {
                CTX.fillText(lines[i], posX, posY + fontSize + (i * fontSize));
            }
        },
        LoadFont: (file_path_ptr) => {
            const buffer = WF.memory.buffer;
            const file_path = getString(buffer, file_path_ptr);

            var id = gen_asset_id();

            console.log("Loading font", { id, file_path });

            // split at the last slash and at the last dot
            // let ext = file_path.split('.').pop();
            let font_name = file_path.split('/').pop().split('.').slice(0, -1).join('.');

            if (FONTS.has(font_name)) {
                // font already loaded
                return FONTS.get(font_name);
            }

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
                FONTS.set(id, font_name);
                return id;
            }).catch((err) => {
                console.log(err);
                return -1;
            });

            return id;
        },
        IsFontLoaded: (font) => {
            return FONTS.has(font);
        },
        DrawTextEx_: (font, text_ptr, posX, posY, fontSize, spacing, color_ptr) => {
            const buffer = WF.memory.buffer;
            const text = getString(buffer, text_ptr);
            const color = getColor(buffer, color_ptr);
            fontSize *= FONT_SCALE_MAGIC;
            CTX.fillStyle = color;

            var font_name = FONTS.get(font);
            if (font_name === undefined) {
                console.log("Font not found", FONTS, font);
                return;
            }

            CTX.font = `${fontSize}px ${font_name}`;

            const lines = text.split('\n');

            for (var i = 0; i < lines.length; i++) {
                const chars = lines[i].split('');
                let x = posX;
                for (var j = 0; j < chars.length; j++) {
                    CTX.fillText(chars[j], x, posY + fontSize + (i * fontSize));
                    x += CTX.measureText(chars[j]).width + spacing;
                }
                // ctx.fillText(lines[i], posX, posY + fontSize + (i * fontSize));
            }
        },
        DrawLine: (startPosX, startPosY, endPosX, endPosY, color_ptr) => {
            const buffer = WF.memory.buffer;
            const color = getColor(buffer, color_ptr);
            CTX.fillStyle = color;
            CTX.beginPath();
            CTX.moveTo(startPosX, startPosY);
            CTX.lineTo(endPosX, endPosY);
            CTX.strokeStyle = color;
            CTX.stroke();
        },
        DrawRectangle: (posX, posY, width, height, color_ptr) => {
            const buffer = WF.memory.buffer;
            const color = getColor(buffer, color_ptr);
            CTX.fillStyle = color;
            CTX.fillRect(posX, posY, width, height);
        },
        DrawRectangleV: (position_ptr, size_ptr, color_ptr) => {
            const buffer = WF.memory.buffer;
            const position = getVector2(buffer, position_ptr);
            const size = getVector2(buffer, size_ptr);
            const color = getColor(buffer, color_ptr);
            CTX.fillStyle = color;
            CTX.fillRect(position.x, position.y, size.x, size.y);
        },
        DrawRectangleRec: (rec_ptr, color_ptr) => {
            const buffer = WF.memory.buffer;
            const rec = getRectangle(buffer, rec_ptr);
            const color = getColor(buffer, color_ptr);
            CTX.fillStyle = color;
            CTX.fillRect(rec.x, rec.y, rec.width, rec.height);
        },
        DrawCircle: (centerX, centerY, radius, color_ptr) => {
            const buffer = WF.memory.buffer;
            const color = getColor(buffer, color_ptr);
            CTX.fillStyle = color;
            CTX.beginPath();
            CTX.arc(centerX, centerY, radius, 0, 2 * Math.PI, 0);
            CTX.fill();
        },
        LoadTexture: (file_path_ptr) => {
            var id = gen_asset_id();
            console.log("Loading texture", { id, file_path });

            const buffer = WF.memory.buffer;
            const file_path = getString(buffer, file_path_ptr);

            let img = new Image();
            TEXTURES[id] = img;
            img.src = file_path;

            return id;
        },
        UnloadTexture: () => {
            drop_asset_id(id);
            delete TEXTURES[id];
        },
        IsTextureLoaded: (id) => {
            const tex = TEXTURES[id];
            if (tex === undefined) {
                return false;
            }
            return TEXTURES[id].complete;
        },
        GetTextureWidth: (id) => {
            const img = TEXTURES[id];
            if (img === undefined) {
                return 0;
            }
            return img.width;
        },
        GetTextureHeight: (id) => {
            const img = TEXTURES[id];
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
            const img = TEXTURES[id];
            CTX.save();
            CTX.translate(x, y);
            CTX.rotate(rotation);
            CTX.scale(scale, scale);
            CTX.drawImage(img, 0, 0);
            CTX.restore();
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
            const img = TEXTURES[id];
            const buffer = WF.memory.buffer;
            const sourceRec = getRectangle(buffer, sourceRec_ptr);
            const destRec = getRectangle(buffer, destRec_ptr);
            CTX.save();
            CTX.translate(destRec.x, destRec.y);
            CTX.drawImage(img, sourceRec.x, sourceRec.y, sourceRec.width, sourceRec.height, 0, 0, destRec.width, destRec.height);
            CTX.restore();
        },
        GetScreenWidth: () => CTX.canvas.width,
        GetScreenHeight: () => CTX.canvas.height,
        GetFrameTime: () => {
            if (TARGET_FPS !== undefined) {
                return Math.min(DT, 1.0 / TARGET_FPS);
            }
            return DT;
        },
        IsWindowResized: () => false,
        WindowShouldClose: () => false,
        SetTargetFPS: (x) => TARGET_FPS = x,
        GetFPS: () => 1.0 / DT,
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
            let msg = getString(ptr);
            console.log(msg);
            window.alert(msg);
        },
        InitAudioDevice: () => { },
        LoadMusicStream: (file_path_ptr) => {
            const buffer = WF.memory.buffer;
            const file_path = getString(buffer, file_path_ptr);

            let id = gen_asset_id();
            console.log("Loading music stream", { id, file_path });

            // Wait for the file fo be fetched
            fetch(file_path).then((response) => {
                console.log(response);
                initAudioContext(response.url);
            }).catch((err) => {
                console.log(err);
            });

            return id;
        },
        UnloadMusicStream: (id) => {
            drop_asset_id(id);
            delete audio[id];
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
            const buffer = WF.memory.buffer;
            const file_path = getString(buffer, file_path_ptr);

            var id = gen_asset_id();
            console.log("Loading image", { id, file_path });

            let img = new Image();

            IMAGES[id] = img;
            img.src = file_path;

            // NOTE: the image is not loaded yet.
            // img.onload = () => console.log("Image loaded", id);

            return id;
        },
        UnloadTexture: (id) => {
            drop_asset_id(id);
            delete TEXTURES[id];
        },
        // pub fn LoadImageColors(image: Image) -> *mut Color;
        // pub struct Color {
        //     pub r: u8,
        //     pub g: u8,
        //     pub b: u8,
        //     pub a: u8,
        // }
        LoadImageColors: (id) => {
            // colors are an array of Color
            const img = IMAGES[id];
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);
            const data = ctx.getImageData(0, 0, img.width, img.height).data;
            const colors = new Uint8Array(WF.memory.buffer, WF.from_js_malloc(data.length), data.length);
            colors.set(data);
            let ptr = colors.byteOffset;
            console.log("Loading image colors", { id, ptr, size: data.length });
            return ptr;
        },
        UnloadImageColors: (ptr, size) => {
            console.log("Unloading image colors", { ptr, size });
            WF.from_js_free(ptr, size);
        },
        IsImageLoaded: (id) => {
            let img = IMAGES[id];
            if (img === undefined) {
                return false;
            }
            return IMAGES[id].complete;
        },
        // pub fn LoadTextureFromImage(image: u32) -> u32;
        LoadTextureFromImage: (id) => {
            var tex_id = gen_asset_id();
            console.log("Loading texture from image", { "image_id": id, "texture_id": tex_id });
            const img = IMAGES[id];
            TEXTURES[tex_id] = img;
            return tex_id;
        },
        // pub fn GetImageWidth(image: u32) -> i32;
        GetImageWidth: (id) => {
            const img = IMAGES[id];
            if (img === undefined) {
                return 0;
            }
            return img.width;
        },
        // pub fn GetImageHeight(image: u32) -> i32;
        GetImageHeight: (image_id) => {
            const img = IMAGES[image_id];
            if (img === undefined) {
                return 0;
            }
            return img.height;
        },
        // pub fn UnloadImage(image: Image) -> ();
        UnloadImage: (image_id) => {
            console.log("Unloading image", image_id);
            delete IMAGES[image_id];
        },
        // pub fn GetTime() -> f64;
        GetTime: () => {
            let t = performance.now();
            return t / 1000;
        }
    })
}).then(w => {
    WASM = w;
    WF = w.instance.exports;
    // console.log(w);

    window.addEventListener("keydown", keyDown);
    window.addEventListener("keyup", keyUp);

    let state = WF.game_init();

    function read_loaded_flag(ptr) {
        const buffer = WASM.instance.exports.memory.buffer;
        var data_view = new DataView(buffer, ptr, 4);
        return data_view.getUint32(0, true) == 1;
    }

    function parse_state(ptr, n_bytes) {
        let schema = "bu[ffff]f{speed}[ff]bu{music}u{font}u{image}u{texture}[uuuu]*u*";
        const buffer = WASM.instance.exports.memory.buffer;
        return wasm_to_struct(buffer, ptr, n_bytes, schema);
    }

    let n_state_size = WF.get_state_size();

    // console.log("State size", n_state_size);
    // console.log("State", parse_state(state, n_state_size));

    const next = (timestamp) => {
        if (QUIT) {
            CTX.clearRect(0, 0, CTX.canvas.width, CTX.canvas.height);
            window.removeEventListener("keydown", keyDown);
            return;
        }
        DT = (timestamp - _PREV_TIMESTAMP) / 1000.0;
        _PREV_TIMESTAMP = timestamp;

        if (read_loaded_flag(state)) {
            WF.game_frame(state);
        } else {
            WF.game_load(state);
            if (read_loaded_flag(state)) {
                console.log("Game loaded!! :D");
                console.log(parse_state(state, n_state_size));
            }
        }
        // log last element of state
        // console.log(parse_state(state, n_state_size));
        window.requestAnimationFrame(next);
        // DEBUG: slow down the loop
        // setTimeout(() => {window.requestAnimationFrame(next);}, 1000);
    };
    window.requestAnimationFrame((timestamp) => {
        _PREV_TIMESTAMP = timestamp;
        window.requestAnimationFrame(next);
    });
}).catch((err) => {
    console.log(err);
    console.log('update WASM_PATH in `main.js` bruv!');
});