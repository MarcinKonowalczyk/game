function cstrlen(mem, ptr) {
    let len = 0;
    while (mem[ptr] != 0) {
        len++;
        ptr++;
    }
    return len;
}

export function getString(buffer, ptr) {
    const mem = new Uint8Array(buffer);
    const len = cstrlen(mem, ptr);
    const bytes = new Uint8Array(buffer, ptr, len);
    return new TextDecoder().decode(bytes);
}

// pub struct Color {
//     pub r: u8,
//     pub g: u8,
//     pub b: u8,
//     pub a: u8,
// }
export function getColor(buffer, ptr) {
    var [r, g, b, a] = new Uint8Array(buffer, ptr, 4);
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
export function getRectangle(buffer, rec_ptr) {
    let mem = new Float32Array(buffer, rec_ptr, 4);
    return { x: mem[0], y: mem[1], width: mem[2], height: mem[3] };
}
// pub struct Vector2 {
//     pub x: f32,
//     pub y: f32,
// }
export function getVector2(buffer, vec_ptr) {
    let mem = new Float32Array(buffer, vec_ptr, 2);
    return { x: mem[0], y: mem[1] };
}