export function wasm_to_struct(buffer, ptr, n_bytes, schema,) {

    var data_view = new DataView(buffer, ptr, n_bytes);

    // let mem = new Uint8Array(buffer, ptr, n_bytes);
    // console.table(mem);

    let tokens = [];
    for (let token of schema_scanner(schema)) {
        tokens.push(token);
    }

    function _to_struct(data_view, tokens, i_offset) {
        i_offset = i_offset || 0;

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
                    _len *= token.value.length;
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
                // console.table(mem);

                let _fun = undefined;

                if (token.type === "struct") {
                    _fun = (dv, k) => _to_struct(_data_view, token.value, k * token.value.length * 4)[0];
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
                    let s = _to_struct(data_view, token.value, i);
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

function parse_until(schema, i, end) {
    let content = "";
    i++;
    while (schema[i] !== end) {
        content += schema[i];
        i++;
    }
    return [content, i];
}

function parse_until_matching(schema, i, start, end) {
    let content = "";
    let depth = 0;
    while (depth >= 0) {
        let char = schema[i];
        if (char === start) {
            depth++;
        } else if (char === end) {
            depth--;
        }
        content += char;
        i++;

        if (i > schema.length) {
            throw "Unexpected end of schema";
        }
    }

    // we've gone too far by one character
    i--;
    content = content.slice(0, -1)

    return [content, i];

}

export function* schema_scanner(schema) {
    let i = 0;
    while (i < schema.length) {
        var out;
        let char = schema[i];
        if (char === "u") {
            out = { type: "uint32", value: char }
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [out.label, i] = parse_until(schema, ++i, "}");
            yield out;
        } else if (char === "f") {
            out = { type: "float32", value: char };
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [out.label, i] = parse_until(schema, ++i, "}");
            yield out;
        } else if (char === "b") {
            out = { type: "bool", value: char };
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [out.label, i] = parse_until(schema, ++i, "}");
            yield out;
        } else if (char === "[") {
            out = { type: "struct" };
            var content = "";
            [content, i] = parse_until_matching(schema, ++i, "[", "]");
            if (schema[i + 1] === "*") { i++; out.is_array = true; }
            if (schema[i + 1] === "{") [out.label, i] = parse_until(schema, ++i, "}");

            // recursively parse the content
            out.value = [];
            for (let token of schema_scanner(content)) {
                out.value.push(token);
            }
            yield out;
        } else if (char === " " || char === "," || char === "\n") {
            // silently skip whitespace and commas and newlines
        } else {
            yield { type: "error", value: char };
        }

        i++;
    }
}
