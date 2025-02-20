export function wasm_to_struct(buffer, ptr, n_bytes, schema,) {

    var data_view = new DataView(buffer, ptr, n_bytes);

    let tokens = [];
    for (let token of schema_scanner(schema)) {
        tokens.push(token);
    }

    function _to_struct(data_view, tokens, offset) {

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
                let len = data_view.getUint32(i + offset, true); i += 4;
                let ptr = data_view.getUint32(i + offset, true); i += 4;

                let _len = len * 4;

                if (token.type === "struct") {
                    _len *= token.value.length;
                }

                if (len === 0) {
                    // empty array or null pointer
                    out.push([]);
                    continue;
                }

                if (ptr === 0) {
                    console.error("Null pointer in array", token);
                    out.push([]);
                    continue;
                }

                let _data_view = new DataView(buffer, ptr, _len);

                let _fun = undefined;

                if (token.type === "struct") {
                    _fun = (dv, j) => _to_struct(_data_view, token.value, j * token.value.length * 4)[0];
                } else if (token.type === "uint32") {
                    _fun = (dv, j) => _data_view.getUint32(j * 4, true);
                } else if (token.type === "float32") {
                    _fun = (dv, j) => _data_view.getFloat32(j * 4, true);
                } else if (token.type === "bool") {
                    _fun = (dv, j) => _data_view.getUint32(j * 4, true) === 1;
                } else {
                    console.error("Unknown token type", token);
                }

                let arr = [];
                for (let j = 0; j < len; j++) {
                    arr.push(_fun(_data_view, j));
                }
                out.push(arr);
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
                    out.push(data_view.getUint32(i + offset) === 1);
                    i += 4;
                } else if (token.type === "struct") {
                    // recursively parse the struct
                    let s = _to_struct(data_view, token.value, i);
                    out.push(s[0]);
                    i += s[1];
                } else {
                    console.error("Unknown token type", token);
                }
            }
        }

        return [out, i];

    }

    let out = {};
    let i = 0;
    [out, i] = _to_struct(data_view, tokens);

    return out;
}

// let schema = "u[ffff]f*{speed}[ff]bu{music}u{font}u{texture}[u{x_max}uuu]*";

function parse_until(schema, i, end) {
    let label = "";
    i++;
    while (schema[i] !== end) {
        label += schema[i];
        i++;
    }
    return [label, i];
}

function* schema_scanner(schema) {
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
            [content, i] = parse_until(schema, i, "]");
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
            i++;
        } else {
            yield { type: "error", value: char };
        }

        i++;
    }
}
