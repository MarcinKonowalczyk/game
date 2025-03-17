'use strict';

import { loopify } from './loopify.js';
import { wasm_to_struct } from './wasm_struct_parser.js';
import { getString, getRectangle, getColor, getVector2 } from './mem_helpers.js';
import { GLFW_MAP } from './glfw_map.js';
import { find_blobs } from './png_font.js';

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


function log_N_times(n) {
    let seen = new Map();
    return (...args) => {
        // get the line number of the caller
        let stack = new Error().stack.split('\n');
        let line = stack[2].split(':').pop();
        let n_called = seen.get(line) || 0;
        seen.set(line, n_called + 1);
        if (n_called > (n - 1)) {
            return;
        }

        console.log(...args);
    }
}

let log_once = log_N_times(1);
let log_head_10 = log_N_times(10);

const GAME = document.getElementById("game");
var CONTAINER = GAME.parentElement; // parent div
const CTX = GAME.getContext("2d");

GAME.keys_state = new Set();
GAME.prev_keys_state = new Set();

let MOUSE_MAP = {
    "Left": 0,
    "Right": 1,
    "Middle": 2, // pressed wheel
    "Side": 3, // advanced mouse device
    "Extra": 4, // advanced mouse device
    "Forward": 5, // advanced mouse device
    "Back": 6, // advanced mouse device
}

GAME.mouseX = -1;
GAME.mouseY = -1;
GAME.mouse_state = new Array(7).fill(false);
GAME.prev_mouse_state = new Array(7).fill(false);

GAME.onmousemove = handleMouseMove;

function handleMouseMove(event) {
    var rect = CONTAINER.getBoundingClientRect();
    var xf = event.offsetX / rect.width;
    var yf = event.offsetY / rect.height;
    GAME.mouseX = xf * GAME.width;
    GAME.mouseY = yf * GAME.height;
}

GAME.onmouseleave = function (event) {
    GAME.mouseX = -1;
    GAME.mouseY = -1;
}

GAME.onmousedown = function (event) {
    GAME.mouse_state[event.button] = true;
}

GAME.onmouseup = function (event) {
    GAME.mouse_state[event.button] = false;
}

GAME.oncontextmenu = function (event) {
    event.preventDefault();
}

window.onkeydown = function (event) {
    event.preventDefault();
    GAME.keys_state.add(event.keyCode);
}

window.onkeyup = function (event) {
    event.preventDefault();
    GAME.keys_state.delete(event.keyCode);
}


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

window.onresize = onResize;

onResize();

if (/iPhone|iPad|iPod|Android/i.test(navigator.userAgent)) {
    // Mobile device style: fill the whole browser client area with the game canvas:
    const meta = document.createElement('meta');
    meta.name = 'viewport';
    meta.content = 'width=device-width, height=device-height, initial-scale=1.0, user-scalable=no, shrink-to-fit=yes';
    document.getElementsByTagName('head')[0].appendChild(meta);
}

let MUSIC_STATUS = {
    "NotFound": -1,
    "NotLoaded": 0,
    "Loaded": 1,
}

let audio = {
    loop: undefined,
    status: MUSIC_STATUS.NotLoaded,
}

function tryToPlayAudio() {
    if (audio.loop === undefined) {
        // no audio
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

let PAUSED = false;
let WASM = undefined;
let DT = undefined;
let WF = undefined;
let QUIT = undefined;
let _PREV_TIMESTAMP = undefined;
let TARGET_FPS = undefined;
let LOG_CALLBACK = undefined;
let LOG_LEVEL = 3; // default to 3=INFO


document.onvisibilitychange = () => PAUSED = document.hidden;

// Add String.format
// https://stackoverflow.com/a/4673436
if (!String.prototype.format) {
    String.prototype.format = function () {
        var args = arguments;
        return this.replace(/{(\d+)}/g, function (match, number) {
            return typeof args[number] != 'undefined'
                ? args[number]
                : match
                ;
        });
    };
}

function wasm_alloc_string(msg) {
    let utf8_encode = new TextEncoder();
    let msg_bytearray = utf8_encode.encode(msg);
    let N = msg_bytearray.length;
    let text_ptr = WF.from_js_malloc(N + 1);
    let text = new Uint8Array(WF.memory.buffer, text_ptr, N + 1);
    text.set(msg_bytearray);
    text[N] = 0; // null-terminated
    return [text_ptr, N];
}

function wasm_free_string(text_ptr, length) {
    WF.from_js_free(text_ptr, length + 1);
}

function _log(level, msg, text_ptr) {
    if (level < LOG_LEVEL) {
        // skip log if below log level
        return;
    }
    if (LOG_CALLBACK !== undefined) {
        var alloced = false;
        if (text_ptr === undefined) {
            var N;
            [text_ptr, N] = wasm_alloc_string(msg);
            alloced = true;
        }
        // NOTE: we pass the pointer, not the text
        // console.log("calling", LOG_CALLBACK, level, text_ptr, WF[LOG_CALLBACK]);
        WF[LOG_CALLBACK](level, text_ptr);
        if (alloced) {
            wasm_free_string(text_ptr, N);
        }
    } else {
        let text = getString(WF.memory.buffer, text_ptr);
        console.log(level, text);
    }
}

const LOG_LEVELS = {
    "ALL": 0,
    "TRACE": 1,
    "DEBUG": 2,
    "INFO": 3,
    "WARNING": 4,
    "ERROR": 5,
    "FATAL": 6,
    "NONE": 999,
}

let info = (msg) => _log(LOG_LEVELS.INFO, msg);
let error = (msg) => _log(LOG_LEVELS.ERROR, msg);

// setup the game RNG
// https://stackoverflow.com/a/47593316
function sfc32(a, b, c, d) {
    return function () {
        a |= 0; b |= 0; c |= 0; d |= 0;
        let t = (a + b | 0) + d | 0;
        d = d + 1 | 0;
        a = b ^ b >>> 9;
        b = c + (c << 3) | 0;
        c = (c << 21 | c >>> 11);
        c = c + t | 0;
        return (t >>> 0) / 4294967296;
    }
}

let _seed = (Math.random() * 2 ** 32) >>> 0;
let rand = sfc32(_seed, _seed, _seed, _seed);
const warmup = () => {
    for (let i = 0; i < 1000; i++) {
        rand();
    }
}

const set_seed = (seed) => {
    rand = sfc32(seed, seed, seed, seed);
    warmup();
}

function load_ttf_font(id, font_name, file_path) {
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
        let font_obj = {
            "name": font_name,
            "kind": "ttf",
            "font": font,
        }
        FONTS.set(id, font_obj);
        return id;
    }).catch((err) => {
        console.log(err);
        return -1;
    });
}

function load_png_font(id, font_name, file_path) {
    if (FONTS.has(font_name)) {
        // font already loaded
        return FONTS.get(font_name);
    }
    console.log("Loading PNG font", { id, font_name, file_path });

    let img = new Image();

    img.onload = () => {
        let blobs = find_blobs(img);
        console.log("Blobs", blobs);
        let font_obj = {
            "name": font_name,
            "kind": "png",
            "img": img,
            "blobs": blobs,
        }
        FONTS.set(id, font_obj);
    }
    img.src = file_path;
}


function foreach_text_ex_ttf(font, text, pos, size, spacing, char_func) {
    let ctx = CTX;
    ctx.font = `${size}px ${font.name}`;

    const lines = text.split('\n');

    for (var i = 0; i < lines.length; i++) {
        const chars = lines[i].split('');
        let x = pos.x;
        for (var j = 0; j < chars.length; j++) {
            let char_info = {
                char: chars[j],
                x: x,
                y: pos.y + size + (i * size),
                size: size,
            }
            char_func(char_info);
            x += ctx.measureText(chars[j]).width + spacing;
        }
    }
}

function draw_text_ex_ttf(font, text, pos, size, spacing, color) {
    let ctx = CTX;
    ctx.fillStyle = color;

    foreach_text_ex_ttf(font, text, pos, size, spacing, (i) => {
        ctx.font = `${i.size}px ${font.name}`;
        ctx.fillText(i.char, i.x, i.y);
    });
}


function measure_text_ex_ttf(font, text, size, spacing) {
    let ctx = CTX;
    ctx.font = `${size}px ${font.name}`;

    let width = 0;
    let height = 0;

    foreach_text_ex_ttf(font, text, { x: 0, y: 0 }, size, spacing, (i) => {
        width = Math.max(width, i.x + ctx.measureText(i.char).width);
        height = Math.max(height, i.y + size);
    });

    return { width, height };
}


function foreach_text_ex_png(font, text, pos, size, spacing, char_func) {
    let blobs = font.blobs;

    const lines = text.split('\n');
    const offset = 32; // offset for the first character (space)

    // determine the width of small 'm' character
    let m_blob = blobs[109 - offset]; // used to determine the width and the base height
    let l_blob = blobs[108 - offset]; // used to determine the ascent
    let j_blob = blobs[106 - offset]; // used to determine the descent

    if (m_blob === undefined || l_blob === undefined || j_blob === undefined) {
        console.error("Missing blobs", { m_blob, l_blob, j_blob });
        return;
    }

    let m_width = m_blob.x_max - m_blob.x_min + 1;
    // let m_height = m_blob.y_max - m_blob.y_min + 1;
    let l_height = l_blob.y_max - l_blob.y_min + 1;
    // let j_height = j_blob.y_max - j_blob.y_min + 1;

    let scale = size / m_width; // how much to uniformly scale the font by

    // let base_height = m_height * scale;
    // let ascender = (l_height - m_height) * scale;
    // let descender = (j_height - m_height) * scale;

    let line_height = l_height * scale + 2;

    for (var i = 0; i < lines.length; i++) {
        const chars = lines[i].split('');
        let x = pos.x;
        for (var j = 0; j < chars.length; j++) {
            let char = chars[j];
            let char_code = char.charCodeAt(0);
            var blob = blobs[char_code - offset];
            if (blob === undefined) {
                // No blob for this character. The last blob is the 'undefined' character
                blob = blobs[blobs.length - 1];
                // console.log("No blob for char", { char, char_code, blob });
            }


            let char_info = {
                char: char,
                sx: blob.x_min,
                sy: blob.y_min,
                sw: blob.x_max - blob.x_min + 1,
                sh: blob.y_max - blob.y_min + 1,
                dx: x,
                dy: pos.y + i * line_height,
                dw: (blob.x_max - blob.x_min + 1) * scale,
                dh: (blob.y_max - blob.y_min + 1) * scale,
            }

            char_func(char_info);
            x += char_info.sw * scale + spacing;
        }
    }
}


function draw_text_ex_png(font, text, pos, size, spacing, color) {
    CTX.fillStyle = color;
    CTX.imageSmoothingEnabled = false;
    foreach_text_ex_png(font, text, pos, size, spacing, (i) => {
        CTX.drawImage(font.img, i.sx, i.sy, i.sw, i.sh, i.dx, i.dy, i.dw, i.dh);
    });
}

function measure_text_ex_png(font, text, size, spacing) {
    let width = 0;
    let height = 0;

    foreach_text_ex_png(font, text, { x: 0, y: 0 }, size, spacing, (i) => {
        width = Math.max(width, i.dx + i.dw);
        height = Math.max(height, i.dy + i.dh);
    });

    return { width, height };
}

let TextFuncs = {
    LoadFont: (file_path_ptr) => {
        const buffer = WF.memory.buffer;
        const file_path = getString(buffer, file_path_ptr);

        var id = gen_asset_id();

        // console.log("Loading font", { id, file_path });
        info("Loading font: id={0}, file_path={1}".format(id, file_path));

        // split at the last slash and at the last dot
        let ext = file_path.split('.').pop();
        let font_name = file_path.split('/').pop().split('.').slice(0, -1).join('.');

        if (ext === "ttf") {
            load_ttf_font(id, font_name, file_path);
        } else if (ext === "png") {
            load_png_font(id, font_name, file_path);
        } else {
            console.error("Unsupported font type", { ext });
        }

        return id;
    },
    IsFontLoaded: (font) => {
        return FONTS.has(font);
    },
    DrawTextEx: (font, text_ptr, position_ptr, fontSize, spacing, color_ptr) => {
        const buffer = WF.memory.buffer;
        const text = getString(buffer, text_ptr);
        const color = getColor(buffer, color_ptr);
        const pos = getVector2(buffer, position_ptr);
        fontSize *= FONT_SCALE_MAGIC;
        var font = FONTS.get(font);
        if (font.name === undefined) {
            console.log("Font not found", FONTS, font);
            return;
        }

        if (font.kind === "ttf") {
            draw_text_ex_ttf(font, text, pos, fontSize, spacing, color);
        } else if (font.kind === "png") {
            draw_text_ex_png(font, text, pos, fontSize, spacing, color);
        } else {
            console.error("Unsupported font kind", { font });
        }
    },
    // pub fn MeasureTextEx(font: Font, text: *const i8, fontSize: i32, spacing: f32) -> Vector2;
    MeasureTextEx: (result_ptr, font, text_ptr, fontSize, spacing) => {
        const buffer = WF.memory.buffer;
        const text = getString(buffer, text_ptr);
        fontSize *= FONT_SCALE_MAGIC;
        var font = FONTS.get(font);
        if (font.name === undefined) {
            console.log("Font not found", FONTS, font);
            return;
        }

        var measure;
        if (font.kind === "ttf") {
            measure = measure_text_ex_ttf(font, text, fontSize, spacing);
        } else if (font.kind === "png") {
            measure = measure_text_ex_png(font, text, fontSize, spacing);
        } else {
            console.error("Unsupported font kind", { font });
        }

        const out = new Float32Array(buffer, result_ptr, 2);
        out[0] = measure.width;
        out[1] = measure.height;
    },
}

let TextureFuncs = {
    LoadTexture: (file_path_ptr) => {
        var id = gen_asset_id();
        // console.log("Loading texture", { id, file_path });
        info("Loading texture: id={0}, file_path={1}".format(id, getString(WF.memory.buffer, file_path_ptr)));

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
    // ffi::GetTextureShape(texture: u32) -> Vector2
    GetTextureShape: (result_ptr, id) => {
        const img = TEXTURES[id];
        const buffer = WF.memory.buffer;
        const result = new Float32Array(buffer, result_ptr, 2);
        result[0] = img.width;
        result[1] = img.height;
    },
    DrawTextureEx: (id, position_ptr, rotation, scale, _color_ptr) => {
        const img = TEXTURES[id];
        const buffer = WF.memory.buffer;
        const position = getVector2(buffer, position_ptr);
        CTX.save();
        CTX.translate(position.x, position.y);
        CTX.rotate(rotation);
        CTX.scale(scale, scale);
        CTX.drawImage(img, 0, 0);
        CTX.restore();
    },
    DrawTexturePro: (id, sourceRec_ptr, destRec_ptr, origin_ptr, rotation_deg) => {
        const img = TEXTURES[id];
        const buffer = WF.memory.buffer;
        const sourceRec = getRectangle(buffer, sourceRec_ptr);
        const destRec = getRectangle(buffer, destRec_ptr);
        const origin = getVector2(buffer, origin_ptr);
        CTX.save();
        CTX.imageSmoothingEnabled = false;

        let scale_x = destRec.width / sourceRec.width;
        let scale_y = destRec.height / sourceRec.height;

        CTX.scale(scale_x, scale_y);

        let angle = rotation_deg / 180 * Math.PI;
        CTX.rotate(angle);

        let tx = destRec.x / scale_x;
        let ty = destRec.y / scale_y;

        CTX.translate(
            tx * Math.cos(angle) + ty * Math.sin(angle),
            ty * Math.cos(angle) - tx * Math.sin(angle),
        )

        CTX.drawImage(img, sourceRec.x, sourceRec.y, sourceRec.width, sourceRec.height, -origin.x / scale_x, -origin.y / scale_y, sourceRec.width, sourceRec.height);
        // CTX.drawImage(img, 0, 0);
        CTX.restore();
    },
    // pub fn LoadImage(file_path: *const i8) -> u32;
    LoadImage: (file_path_ptr) => {
        const buffer = WF.memory.buffer;
        const file_path = getString(buffer, file_path_ptr);

        var id = gen_asset_id();
        // console.log("Loading image", { id, file_path });
        info("Loading image: id={0}, file_path={1}".format(id, file_path));

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
    LoadImageColors: (id, result_ptr) => {
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
        // console.log("Loading image colors", { id, ptr, size: data.length });
        info("Loading image colors: id={0}, ptr={1}, size={2}".format(id, ptr, data.length));
        return ptr;
    },
    UnloadImageColors: (ptr, size) => {
        // console.log("Unloading image colors", { ptr, size });
        info("Unloading image colors: ptr={0}, size={1}".format(ptr, size));
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
        // console.log("Loading texture from image", { "image_id": id, "texture_id": tex_id });
        info("Loading texture from image: image_id={0}, texture_id={1}".format(id, tex_id));
        const img = IMAGES[id];
        TEXTURES[tex_id] = img;
        return tex_id;
    },
    // pub fn GetImageShape(image: u32) -> Vector2;
    GetImageShape: (result_ptr, id) => {
        const img = IMAGES[id];
        const buffer = WF.memory.buffer;
        const result = new Float32Array(buffer, result_ptr, 2);
        result[0] = img.width;
        result[1] = img.height;
    },
    // pub fn UnloadImage(image: Image) -> ();
    UnloadImage: (image_id) => {
        // console.log("Unloading image", image_id);
        info("Unloading image: id={0}".format(image_id));
        delete IMAGES[image_id];
    },
}

let LogFuncs = {
    ConsoleLog: (text_ptr, args_ptr) => {
        let buffer = WF.memory.buffer;
        let args = new Array();
        if (args_ptr !== 0) {
            let offset = 0;
            const special = "<END>";
            while (true) {
                let next_arg = getString(buffer, args_ptr + offset);
                if (next_arg === special) break;
                args.push(next_arg);
                offset += next_arg.length + 1;
            }
        }
        if (args.length === 0) {
            console.log(getString(buffer, text_ptr));
        } else {
            console.log(getString(buffer, text_ptr), ...args);
        }
    },
    Log: (level, text_ptr) => _log(level, "", text_ptr),
    // pub fn SetTraceLogCallback(callback_name: *const i8) -> ();
    SetTraceLogCallback: (callback_name_ptr) => {
        const buffer = WF.memory.buffer;
        var func_name = getString(buffer, callback_name_ptr);
        let parts = func_name.split('::');
        func_name = parts.pop();

        // check if we have a function with that name in WF
        if (func_name === "") {
            // unset the callback
            // console.log("Unsetting logging callback", { func_name });
            info("Unsetting logging callback: func_name={0}".format(func_name));
            LOG_CALLBACK = undefined;
        } else if (WF[func_name] === undefined) {
            console.error("Function not found", { func_name });
        } else {
            // console.log("Setting logging callback", { func_name });
            LOG_CALLBACK = func_name
            info("Setting logging callback: func_name={0}".format(func_name));
        }
    },
    SetTraceLogLevel: (level) => {
        LOG_LEVEL = level;
    },
}

let InterfaceFuncs = {
    GetMousePosition: (result_ptr) => {
        const buffer = WF.memory.buffer;
        const result = new Float32Array(buffer, result_ptr, 2);
        result[0] = GAME.mouseX;
        result[1] = GAME.mouseY;
    },
    IsMouseButtonDown: (button) => GAME.mouse_state[button],
    IsMouseButtonPressed: (button) => GAME.mouse_state[button] && !GAME.prev_mouse_state[button],
    IsKeyDown: (key) => GAME.keys_state.has(key),
    IsKeyPressed: (key) => GAME.keys_state.has(key) && !GAME.prev_keys_state.has(key),
    IsKeyReleased: (key) => GAME.prev_keys_state.has(key) && !GAME.keys_state.has(key),
}

let AudioFuncs = {
    InitAudioDevice: () => { },
    LoadMusicStream: (file_path_ptr) => {
        const buffer = WF.memory.buffer;
        const file_path = getString(buffer, file_path_ptr);

        let id = gen_asset_id();
        // console.log("Loading music stream", { id, file_path });
        info("Loading music stream: id={0}, file_path={1}".format(id, file_path));

        // Wait for the file fo be fetched
        fetch(file_path).then((response) => {
            loopify(response.url, function (err, loop) {
                if (err) {
                    error(err);
                    audio.status = MUSIC_STATUS.NotFound;
                } else {
                    audio.loop = loop;
                    audio.status = MUSIC_STATUS.Loaded;
                }
            });
        });

        return id;
    },
    UnloadMusicStream: (id) => {
        drop_asset_id(id);
        delete audio[id];
    },
    MusicStatus: (id) => audio.status,
    PlayMusicStream: (_audio_id) => {
        tryToPlayAudio();
    },
    UpdateMusicStream: (_audio_id) => {
        tryToPlayAudio();
    },
    SetMusicVolume: (_audio_id, volume) => {
        if (audio.loop === undefined) {
            return;
        }
        audio.loop.volume(volume);
    },
}

let OtherFuncs = {
    InitWindow: (width, height, title_ptr) => {
        let title = getString(WF.memory.buffer, title_ptr);
        // console.log("InitWindow", { width, height, title });
        info("InitWindow: width={0}, height={1}, title={2}".format(width, height, title));
        GAME.width = width;
        GAME.height = height;
        document.title = title;
    },
    BeginDrawing: () => { },
    CloseWindow: () => { },
    EndDrawing: () => { },
    ClearBackground: (color_ptr) => {
        const buffer = WF.memory.buffer;
        const color = getColor(buffer, color_ptr);
        CTX.fillStyle = color;
        CTX.fillRect(0, 0, CTX.canvas.width, CTX.canvas.height);
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
    GetScreenShape: (result_ptr) => {
        const buffer = WF.memory.buffer;
        const result = new Float32Array(buffer, result_ptr, 2);
        result[0] = CTX.canvas.width;
        result[1] = CTX.canvas.height;
    },
    GetFrameTime: () => {
        if (TARGET_FPS !== undefined) return Math.min(DT, 1.0 / TARGET_FPS);
        if (DT === undefined) return 0.0;
        return DT;
    },
    IsWindowResized: () => false,
    WindowShouldClose: () => false,
    SetTargetFPS: (x) => TARGET_FPS = x,
    // GetFPS: () => 1.0 / DT,
    GetFPS: () => {
        if (DT === undefined) return 0.0;
        return 1.0 / DT;
    },
    // pub fn GetTime() -> f64;
    GetTime: () => {
        let t = performance.now();
        return t / 1000;
    },
    // pub fn DrawLineEx(startPos: Vector2, endPos: Vector2, thickness: f32, color: *const Color);
    DrawLineEx(startPos_ptr, endPos_ptr, thickness, color_ptr) {
        const buffer = WF.memory.buffer;
        const startPos = getVector2(buffer, startPos_ptr);
        const endPos = getVector2(buffer, endPos_ptr);
        const color = getColor(buffer, color_ptr);
        CTX.beginPath();
        CTX.moveTo(startPos.x, startPos.y);
        CTX.lineTo(endPos.x, endPos.y);
        CTX.lineWidth = thickness;
        CTX.strokeStyle = color;
        CTX.stroke();
        CTX.closePath();
        CTX.lineWidth = 1;
    },
    // pub fn SetRandomSeed(seed: u32);
    SetRandomSeed: (seed) => set_seed(seed),
    // pub fn GetRandomValue(min: i32, max: i32) -> i32
    GetRandomValue(min, max) {
        return Math.floor(rand() * (max - min + 1) + min);
    }
};

WebAssembly.instantiateStreaming(fetch(WASM_PATH), {
    "env": make_environment(TextFuncs, TextureFuncs, LogFuncs, InterfaceFuncs, AudioFuncs, OtherFuncs)
}).then(w => {
    WASM = w;
    WF = w.instance.exports;
    console.log(w);

    // window.addEventListener("keydown", keyDown);
    // window.addEventListener("keyup", keyUp);

    let state = WF.game_init();

    function read_loaded_flag(ptr) {
        const buffer = WASM.instance.exports.memory.buffer;
        var data_view = new DataView(buffer, ptr, 4);
        return data_view.getUint32(0, true) == 1;
    }

    function parse_state(ptr, n_bytes) {
        // let schema = 'b{all_loaded}f{curr_time}f{prev_time}u{frame_count}[f{x}f{y}f{width}f{height}]{rect}[f{x}f{y}]{mouse_pos}b{mouse_btn}b{mouse_btn_pressed}u{music}u{font}u{image}u{texture}[u{x_min}u{y_min}u{x_max}u{y_max}]*{anim_blobs}[f{x}f{y}]*{path}f{path_length}[f{position}f{health}f{max_health}f{spawn_time}f{last_hit_time}b{dead}]*{enemies}b{mute}[[f{x}f{y}]{position}b{dead}]*{turrets}';
        let schema = `
          b{all_loaded}
          f{curr_time}
          f{prev_time}
          u{frame_count}
          [f{x}f{y}]{slime_pos}
          [f{x}f{y}]{mouse_pos}
          b{mouse_btn}
          b{mouse_btn_pressed}
          u{music}
          u{font}
          u{image}
          u{texture}
          [uuuu]*{anim_blobs}
          [f{x}f{y}]*{path}
          f{path_length}
          b{mute}
          u{life}
          `;
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

        if (!PAUSED) {
            DT = (timestamp - _PREV_TIMESTAMP) / 1000.0;
            _PREV_TIMESTAMP = timestamp;

            if (read_loaded_flag(state)) {
                WF.game_frame(state);
            } else {
                WF.game_load(state);
                if (read_loaded_flag(state)) {
                    console.log("Game loaded!! :D");
                    try {
                        let parsed_state = parse_state(state, n_state_size);
                        console.log(parsed_state);
                    } catch (e) {
                        console.log(e);
                    }
                }
            }

            // state history between frames
            GAME.prev_mouse_state = GAME.mouse_state.slice();
            GAME.prev_keys_state = new Set(GAME.keys_state);
        }

        // log last element of state
        // let parsed_state = parse_state(state, n_state_size);
        // let dt = parsed_state.curr_time - parsed_state.prev_time;
        // console.log(parsed_state.enemies[0]);
        window.requestAnimationFrame(next);
        // setTimeout(() => { window.requestAnimationFrame(next); }, 0);
        // DEBUG: slow down the loop
        // setTimeout(() => {window.requestAnimationFrame(next);}, 1000);
    };
    _PREV_TIMESTAMP = performance.now();
    next(performance.now());
}).catch((err) => {
    console.log(err);
    console.log('update WASM_PATH in `main.js` bruv!');
});