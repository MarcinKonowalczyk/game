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

    assert_eq(token.type, "struct");
    assert_eq(token.value.length, 2);

    let [x, y] = token.value;

    assert_eq(x.type, "float32");
    assert_eq(x.value, "f");
    assert_eq(x.label, "x");

    assert_eq(y.type, "float32");
    assert_eq(y.value, "f");
    assert_eq(y.label, "y");
}

TESTS.array = () => {
    let tokens = yield_tokens('f*{speed}')

    assert_eq(tokens.length, 1);
    assert_eq(tokens[0].type, "float32");
    assert_eq(tokens[0].value, "f");
    assert_eq(tokens[0].label, "speed");
    assert_eq(tokens[0].is_array, true);
}

TESTS.skip_me = () => {
    skip("This test is skipped");
}

////////////////////////////////////////
// Run tests
////////////////////////////////////////

let OUTPUT = [];
let ASSERTION_NO = 0;

let LOG = process.stdout.write.bind(process.stdout);
let _console_log = console.log;
console.log = function () {
    OUTPUT.push(Array.from(arguments));
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