#!/usr/bin/env node

import { schema_scanner } from "./wasm_struct_parser.js"

////////////////////////////////////////
// Helper functions
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

////////////////////////////////////////
// Tests
////////////////////////////////////////

function yield_tokens(schema) {
    let tokens = [];
    for (let token of schema_scanner(schema)) {
        tokens.push(token);
    }
    return tokens;
}

let TESTS = {};

TESTS.f32 = () => {
    let tokens = yield_tokens('f{time}')

    assert_eq(tokens.length, 1);
    assert_eq(tokens[0].type, "float32");
    assert_eq(tokens[0].value, "f");
    assert_eq(tokens[0].label, "time");
}

TESTS.u32 = () => {
    let tokens = yield_tokens('u{font}')

    assert_eq(tokens.length, 1);
    assert_eq(tokens[0].type, "uint32");
    assert_eq(tokens[0].value, "u");
    assert_eq(tokens[0].label, "font");
}


TESTS.boolean = () => {
    let tokens = yield_tokens('b{dead}')

    assert_eq(tokens.length, 1);
    assert_eq(tokens[0].type, "bool");
    assert_eq(tokens[0].value, "b");
    assert_eq(tokens[0].label, "dead");
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


for (let test in TESTS) {
    LOG(`Running test: ${test} `);

    var result = 'failed';
    var err = null;

    try {
        TESTS[test]();
        result = 'passed';
    } catch (e) {
        err = e;
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
        LOG("Output:");
        for (let o of OUTPUT) {
            // recursively print arrays
            LOG(" ", o[0]);
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