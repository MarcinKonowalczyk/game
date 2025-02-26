#!/usr/bin/env node

import { schema_scanner } from "./wasm_struct_parser.js"

////////////////////////////////////////
// Test Framework Helper functions
////////////////////////////////////////

function assert(condition, message) {
    if (!condition) {
        throw message || "Assertion failed";
    }
    ASSERTION_NO++;
}

function assert_eq(a, b) {
    if (a !== b) {
        throw `Expected ${a} to equal ${b}. Assertion ${ASSERTION_NO}`;
    }
    ASSERTION_NO++;
}

function skip(reason) {
    throw `Skipped: ${reason}`;
}

let TESTS = {};

////////////////////////////////////////
// Test-specific Helper functions
////////////////////////////////////////

function yield_tokens(schema) {
    let tokens = [];
    for (let token of schema_scanner(schema)) {
        tokens.push(token);
    }
    return tokens;
}

function yield_token(schema) {
    let tokens = yield_tokens(schema);
    assert_eq(tokens.length, 1);
    return tokens[0];
}

// Helper function to make sure parsing of [f{x}f{y}] struct is correct
function assert_vec2(token) {
    assert_eq(token.type, "struct");
    assert_eq(token.value.length, 2);

    let [x_token, y_token] = token.value;

    assert_eq(x_token.type, "float32");
    assert_eq(x_token.value, "f");
    assert_eq(x_token.label, "x");

    assert_eq(y_token.type, "float32");
    assert_eq(y_token.value, "f");
    assert_eq(y_token.label, "y");
}

////////////////////////////////////////
// Tests
////////////////////////////////////////

TESTS.f32 = () => {
    let token = yield_token('f{time}')

    assert_eq(token.type, "float32");
    assert_eq(token.value, "f");
    assert_eq(token.label, "time");
}

TESTS.u32 = () => {
    let token = yield_token('u{font}')

    assert_eq(token.type, "uint32");
    assert_eq(token.value, "u");
    assert_eq(token.label, "font");
}


TESTS.boolean = () => {
    let token = yield_token('b{dead}')

    assert_eq(token.type, "bool");
    assert_eq(token.value, "b");
    assert_eq(token.label, "dead");
}

TESTS.struct = () => {
    let token = yield_token('[f{x}f{y}]')

    assert_vec2(token);
}

TESTS.array = () => {
    let tokens = yield_tokens('f*{speed}')

    assert_eq(tokens.length, 1);
    assert_eq(tokens[0].type, "float32");
    assert_eq(tokens[0].value, "f");
    assert_eq(tokens[0].label, "speed");
    assert_eq(tokens[0].is_array, true);
}

TESTS.nested_struct = () => {
    let tokens = yield_tokens('[[f{x}f{y}][f{x}f{y}]]')

    assert_eq(tokens.length, 1);
    assert_eq(tokens[0].type, "struct");
    assert_eq(tokens[0].value.length, 2);

    let [a, b] = tokens[0].value;

    assert_vec2(a);
    assert_vec2(b);
}

TESTS.array_def_with_spaces = () => {
    let tokens = yield_tokens('f     *{my_array}')

    assert_eq(tokens.length, 1);
    assert_eq(tokens[0].type, "float32");
    assert_eq(tokens[0].value, "f");
    assert_eq(tokens[0].label, "my_array");
    assert_eq(tokens[0].is_array, true);
}

TESTS.preserve_spaces_in_label = () => {
    let token = yield_token('f{time elapsed}')

    assert_eq(token.type, "float32");
    assert_eq(token.value, "f");
    assert_eq(token.label, "time elapsed");
}

TESTS.strip_label_spaces = () => {
    let token = yield_token('f{  time elapsed  }')

    assert_eq(token.type, "float32");
    assert_eq(token.value, "f");
    assert_eq(token.label, "time elapsed");
}

TESTS.spaces_everywhere = () => {
    // spaces everywhere
    let token = yield_token('  [  f{x}  f{y}  ]  *  {path}  ')

    assert_vec2(token);
    assert_eq(token.is_array, true);
    assert_eq(token.label, "path");
}

TESTS.skip_me = () => {
    skip("This test is skipped");
}

////////////////////////////////////////
// Argument parsing
////////////////////////////////////////

let args = process.argv.slice(2);

function _match_flag(args, flags) {
    let indices = flags.map(f => args.indexOf(f)).filter(i => i !== -1);
    if (indices.length === 0) {
        return -1; // not found
    } else if (indices.length > 1) {
        throw `Only one flag is allowed from ${flags}`;
    }
    return indices[0];
}

function match_flag(args, flags, default_) {
    default_ == default_ || false;
    let index = _match_flag(args, flags);
    if (index === -1) {
        return default_;
    }
    args.splice(index, 1);
    return !default_;
}

function match_flag_and_value(args, flags, default_) {
    default_ == default_ || null;
    let index = _match_flag(args, flags);
    if (index === -1) {
        return default_;
    }
    if (index + 1 >= args.length) {
        throw `Flag ${flags[0]} requires a value`;
    }
    let value = args[index + 1];
    args.splice(index, 2);
    return value;
}

let ARGS = {};

try {
    ARGS.verbose = match_flag(args, ["-v", "--verbose"]);
    ARGS.filter = match_flag_and_value(args, ["-k", "--filter"], '')
    ARGS.capture = match_flag(args, ["-s", "--capture=no"], true);
} catch (e) {
    console.log(e);
    process.exit(1);
}

// console.log("Arguments: ", ARGS);
// process.exit(1);

if (args.length > 0) {
    console.log("Unknown arguments: ", args);
    process.exit(1);
}

////////////////////////////////////////
// Run tests
////////////////////////////////////////

if (ARGS.filter !== '') {
    if (ARGS.verbose) {
        console.log("Filtering tests with: ", ARGS.filter);
    }

    let filtered_tests = {};
    for (let test in TESTS) {
        if (test.includes(ARGS.filter)) {
            filtered_tests[test] = TESTS[test];
        }
    }
    TESTS = filtered_tests;
}

let OUTPUT = [];
let ASSERTION_NO = 0;

let LOG = process.stdout.write.bind(process.stdout);
let _console_log = console.log;
console.log = function () {
    if (ARGS.capture) {
        // capture the output
        OUTPUT.push(Array.from(arguments));
    } else {
        // call the original console.log
        _console_log.apply(console, arguments);
    }
}

const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const RED = '\x1b[31m';
const RESET = '\x1b[0m';

let STATS = {
    passed: 0,
    failed: 0,
    skipped: 0
};

function arguments_to_string(args) {
    let s = "";
    for (let i = 0; i < args.length; i++) {
        s += JSON.stringify(args[i], null).replace(/"/g, "").replace(/,/g, ", ");
        if (i < args.length - 1) {
            s += " -- ";
        }
    }
    return s;
}

for (let test in TESTS) {
    LOG(`Test: ${test} `);

    var result = 'failed';
    var err = null;

    try {
        TESTS[test]();
        result = 'passed';
    } catch (e) {
        err = `${e}`;
        result = 'failed';
    }

    if (err != null && err.startsWith("Skipped")) {
        result = 'skipped';
    }

    if (result === 'skipped') {
        LOG(`${YELLOW}skipped${RESET}`);
        STATS.skipped++;
    } else if (result === 'failed') {
        LOG(`${RED}failed: ${err}${RESET}`);
        STATS.failed++;
    } else if (result === 'passed') {
        LOG(`${GREEN}passed${RESET}`);
        STATS.passed++;
    }

    LOG("\n");

    if (OUTPUT.length > 0) {
        LOG("Output:\n");
        for (let o of OUTPUT) {
            LOG(" ")
            LOG(arguments_to_string(o));
            LOG("\n")
        }
    }

    OUTPUT = [];
    ASSERTION_NO = 0;
}

console.log = _console_log;

// print stats
let total = STATS.passed + STATS.failed + STATS.skipped;
let skipped_color = STATS.skipped > 0 ? YELLOW : GREEN;
let failed_color = STATS.failed > 0 ? RED : GREEN;

LOG(`Run ${total} tests: ${GREEN}${STATS.passed} passed${RESET}, ${skipped_color}${STATS.skipped} skipped${RESET}, ${failed_color}${STATS.failed} failed${RESET}`);

if (STATS.failed > 0) {
    process.exit(1);
} else {
    process.exit(0);
}