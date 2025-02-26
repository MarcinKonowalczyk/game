function value_length(token) {
    if (typeof token.value !== "object") {
        console.error("Invalid token in value_length", token);
        return -1;
    }
    let len = 0;
    for (let t of token.value) {
        if (t.is_array) {
            // token is an array. will be length-prefixed
            len += 2;
            continue;
        }

        if (t.type === "struct") {
            len += value_length(t);
        } else if (t.type === "uint32" || t.type === "float32" || t.type === "bool") {
            len += 1;
        } else {
            console.error("Unknown token type", t);
        }
    }
    return len;
}

export function wasm_to_struct(buffer, ptr, n_bytes, schema,) {

    var data_view = new DataView(buffer, ptr, n_bytes);

    // let mem = new Uint8Array(buffer, ptr, n_bytes);
    // console.table(mem);

    let tokens = [];
    for (let token of schema_scanner(schema)) {
        tokens.push(token);
    }

    for (let token of tokens) {
        if (token.type === "error") {
            console.error("Error parsing schema", token);
            return;
        }
    }

    function _to_struct(data_view, tokens, i_offset) {
        i_offset = i_offset || 0;
        // if (i_offset != 0) console.log("Offset", i_offset);

        var out = [];
        let i = 0; // byte index
        let j = 0; // token index
        for (let token of tokens) {

            if (token.type === "error") {
                console.error("Error parsing schema", token);
                return;
            }

            if (token.label == undefined) {
                token.label = j.toString();
            }

            if (token.is_array) {
                let len = data_view.getUint32(i + i_offset, true); i += 4;
                let ptr = data_view.getUint32(i + i_offset, true); i += 4;

                let _len = len * 4;

                if (token.type === "struct") {
                    _len *= value_length(token);
                }

                if (len === 0) {
                    // empty array or null pointer
                    out.push([token.label, []]);
                    continue;
                }

                if (ptr === 0) {
                    console.error("Null pointer in array", token);
                    out.push([token.label, []]);
                    continue;
                }

                let _data_view = new DataView(buffer, ptr, _len);

                // let mem = new Uint8Array(buffer, ptr, _len);
                // console.log("Array", token.label, len, ptr, _len, token.value);
                // console.table(mem);

                let _fun = undefined;

                if (token.type === "struct") {
                    _fun = (dv, k) => _to_struct(_data_view, token.value, k * value_length(token) * 4)[0];
                } else if (token.type === "uint32") {
                    _fun = (dv, k) => _data_view.getUint32(k * 4, true);
                } else if (token.type === "float32") {
                    _fun = (dv, k) => _data_view.getFloat32(k * 4, true);
                } else if (token.type === "bool") {
                    _fun = (dv, k) => _data_view.getUint32(k * 4, true) === 1;
                } else {
                    console.error("Unknown token type", token);
                }

                let arr = [];
                for (let k = 0; k < len; k++) {
                    arr.push(_fun(_data_view, k));
                }
                out.push([token.label, arr]);
            } else {
                // parse single token
                if (token.type === "uint32") {
                    out.push([token.label, data_view.getUint32(i + i_offset, true)]);
                    i += 4;
                } else if (token.type === "float32") {
                    out.push([token.label, data_view.getFloat32(i + i_offset, true)]);
                    i += 4;
                } else if (token.type === "bool") {
                    // We are 4-byte aligned, so a bool takes 4 bytes
                    out.push([token.label, data_view.getUint32(i + i_offset, true) === 1]);
                    i += 4;
                } else if (token.type === "struct") {
                    // recursively parse the struct
                    let s = _to_struct(data_view, token.value, i + i_offset);
                    out.push([token.label, s[0]]);
                    i += s[1];
                } else {
                    console.error("Unknown token type", token);
                }
            }

            j += 1;
        }

        var out2 = {};
        for (let e of out) {
            out2[e[0]] = e[1];
        }

        return [out2, i];

    }

    let out = {};
    let i = 0;
    [out, i] = _to_struct(data_view, tokens);

    return out;
}

// let schema = "u[ffff]f*{speed}[ff]bu{music}u{font}u{texture}[u{x_max}uuu]*";

function parse_until(s, end) {
    let content = "";
    s.i++;
    while (s.schema[s.i] !== end) {
        content += s.schema[s.i];
        s.i++;
    }
    return content;
}

function parse_matching(s, delimiters) {
    if (delimiters.length !== 2) {
        throw "Invalid delimiters"
    }
    let [start, end] = delimiters;

    if (s.schema[s.i] !== delimiters[0]) {
        throw `Expected start delimiter ${delimiters[0]} but got ${s.schema[s.i]}`;
    }
    s.i++;

    let content = "";
    let depth = 0;

    while (depth >= 0) {
        let char = s.schema[s.i];
        if (char === start) {
            depth++;
        } else if (char === end) {
            depth--;
        }
        content += char;
        s.i++;

        if (s.i > s.schema.length) {
            throw "Unexpected end of schema";
        }
    }

    // we've gone too far by one character
    s.i--;
    content = content.slice(0, -1)

    return content;
}

let IGNORE = [" ", ",", "\n"];

// skip until the next non-ignored character
function skip_ignored(s) {
    while (s.i < s.schema.length && IGNORE.includes(s.schema[s.i + 1])) {
        s.i++;
    }
}

// try to match the next character in the schema. if it matches, advance the index
function match_next(s, char) {
    skip_ignored(s);
    if (s.schema[s.i + 1] === char) {
        s.i++;
        return true;
    }
    return false;
}

export function* schema_scanner(schema_) {
    let s = { i: 0, schema: schema_ }; // state
    while (s.i < s.schema.length) {
        var out;
        let char = s.schema[s.i];
        if (char === "u") {
            out = { type: "uint32", value: char }
            if (match_next(s, "*")) { out.is_array = true; }
            if (match_next(s, "{")) { out.label = parse_matching(s, "{}").trim(); }
            yield out;
        } else if (char === "f") {
            out = { type: "float32", value: char };
            if (match_next(s, "*")) { out.is_array = true; }
            if (match_next(s, "{")) { out.label = parse_matching(s, "{}").trim(); }
            yield out;
        } else if (char === "b") {
            out = { type: "bool", value: char };
            if (match_next(s, "*")) { out.is_array = true; }
            if (match_next(s, "{")) { out.label = parse_matching(s, "{}").trim(); }
            yield out;
        } else if (char === "[") {
            out = { type: "struct" };
            let content = parse_matching(s, "[]");
            if (match_next(s, "*")) { out.is_array = true; }
            if (match_next(s, "{")) { out.label = parse_matching(s, "{}").trim(); }
            out.value = [...schema_scanner(content)];
            yield out;
        } else if (IGNORE.includes(char)) {
            // silently skip whitespace and commas and newlines
        } else {
            yield { type: "error", value: char };
        }

        s.i++;
    }
}
