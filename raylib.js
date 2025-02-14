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

function color_hex_unpacked(r, g, b, a) {
    r = r.toString(16).padStart(2, '0');
    g = g.toString(16).padStart(2, '0');
    b = b.toString(16).padStart(2, '0');
    a = a.toString(16).padStart(2, '0');
    return "#" + r + g + b + a;
}

function getColorFromMemory(buffer, color_ptr) {
    const [r, g, b, a] = new Uint8Array(buffer, color_ptr, 4);
    return color_hex_unpacked(r, g, b, a);
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

let images = []

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
        // DrawTexture: (id, x, y, color_ptr) => {
        //     console.log(x, y, id);
        //     const img = images[id];
        //     ctx.drawImage(img, 0, y);
        // },
        LoadTexture: (result_ptr, file_path_ptr) => {
            const buffer = wf.memory.buffer;
            const file_path = cstr_by_ptr(buffer, file_path_ptr);

            let result = new Uint32Array(buffer, result_ptr, 5)
            let img = new Image();
            img.src = file_path;
            images.push(img);

            img.onload = () => {
                images.push(img);
                result[0] = images.indexOf(img);
                result[1] = img.width; // width
                result[2] = img.height; // height
                result[3] = 1; // mipmaps
                result[4] = 7; // format PIXELFORMAT_UNCOMPRESSED_R8G8B8A8
            };

            return result;
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

            // Wait for the file fo be fetched
            fetch(file_path).then((response) => {
                console.log(response);
                initAudioContext(response.url);
                let audio = new Audio();
                return -1;
            }).catch((err) => {
                console.log(err);
                return -1;
            });
        },
        PlayMusicStream: (_audio_id) => {
            tryToPlayAudio();
        },
        UpdateMusicStream: (_audio_id) => {
            tryToPlayAudio();
        },
    })
}).then(w => {
    wasm = w;
    wf = w.instance.exports;
    console.log(w);

    window.addEventListener("keydown", keyDown);
    window.addEventListener("keyup", keyUp);

    let state = wf.game_init();
    const next = (timestamp) => {
        if (quit) {
            ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
            window.removeEventListener("keydown", keyDown);
            return;
        }
        dt = (timestamp - prev) / 1000.0;
        prev = timestamp;
        wf.game_frame(state);
        window.requestAnimationFrame(next);
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