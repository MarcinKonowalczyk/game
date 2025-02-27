#!/usr/bin/env python3

__version__ = "0.2.0"

try:
    import colorama  # type: ignore

    colorama.init()
except ImportError:
    colorama = None


def cprint(message: str, color: str | None = None) -> None:
    """Print a message in color. If colorama is not installed, the message will be printed without color."""
    if colorama is not None and color is not None:
        print(
            colorama.Fore.__dict__[color.upper()] + message + colorama.Style.RESET_ALL
        )
    else:
        print(message)


def parse_rust_structs(content: str) -> dict[str, list[dict[str, str]]]:
    """Parse a rust file and return a dictionary of structs and their fields."""

    # remove comments
    import re

    content = re.sub(r"//.*", "", content)
    content = re.sub(r"/\*.*?\*/", "", content, flags=re.DOTALL)

    # find all structs
    structs = re.findall(r"struct\s+(\w+)\s*{([^}]*)}", content)

    # find all fields in each struct
    out = {}
    for name, field_lines in structs:
        fields = []
        for line in field_lines.split("\n"):
            # # a line is maybe a 'pub' keyword, a field name, and a type, and maybe a comma
            match = re.match(
                r"\s*(?P<pub>pub)?\s*(?P<name>\w+)\s*:\s*(?P<ptr>(?:\*const)|(?:\*mut)?)\s*(?P<type>[^,]+),?",
                line,
            )
            if not match:
                continue

            d: dict[str, str] = match.groupdict()
            fields.append(d)

        out[name] = fields

    return out


def struct_to_schema(
    struct: list[dict[str, str]], env: dict[str, list[dict[str, str]]]
) -> str:
    """Convert a struct to a schema."""

    schema = ""

    struct_next = struct[1:] + [None]

    i = 0
    while i < len(struct):
        field = struct[i]
        next_field = struct_next[i]

        name = field["name"]

        if (
            name[-2:] == "_n"
            and next_field
            and next_field["name"][-4:] == "_arr"
            and next_field["ptr"] != ""
        ):
            name = name[:-2]
            next_name = next_field["name"][:-4]
            if name != next_name:
                raise ValueError(
                    f"Array field names do not match: {name} != {next_name}"
                )

            if next_field["type"] == "bool":
                schema += f"b*{{{name}}}"
            elif next_field["type"] == "f32":
                schema += f"f*{{{name}}}"
            elif next_field["type"] == "u32":
                schema += f"u*{{{name}}}"
            else:
                # not a primitive type. look it up in the environment
                value = env.get(
                    next_field["type"], env.get(next_field["type"].split("::")[-1])
                )
                if value:
                    if isinstance(value, str):
                        # we have a value coming from a mapping
                        schema += f"{value}*{{{name}}}"
                    elif isinstance(value, list):
                        # we have a struct coming from the environment
                        schema += f"[{struct_to_schema(value, env)}]*{{{name}}}"
                    else:
                        raise NotImplementedError("1")
                else:
                    raise ValueError(
                        f"Type {next_field['type']} not implemented as a primitive type, and not found in environment."
                    )

            i += 1
        else:
            if field["type"] == "bool":
                schema += f"b{{{name}}}"
            elif field["type"] == "f32":
                schema += f"f{{{name}}}"
            elif field["type"] == "u32":
                schema += f"u{{{name}}}"
            else:
                # not a primitive type. look it up in the environment
                value = env.get(field["type"], env.get(field["type"].split("::")[-1]))
                if value:
                    if isinstance(value, str):
                        # we have a value coming from a mapping
                        schema += f"{value}{{{name}}}"
                    elif isinstance(value, list):
                        # we have a struct coming from the environment
                        schema += f"[{struct_to_schema(value, env)}]{{{name}}}"
                    else:
                        raise ValueError(
                            f"Type {field['type']} not an implemented primitive type or found in environment."
                        )
                else:
                    raise ValueError(
                        f"(2) Type {field['type']} not an implemented primitive type or found in environment."
                    )

        i += 1

    return schema


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Parse a rust game State to a javascript schema."
    )

    parser.add_argument(
        "-v",
        "--verbose",
        help="Print verbose output.",
        action="store_true",
        default=False,
    )

    parser.add_argument(
        "--version",
        action="version",
        version=f"%(prog)s {__version__}",
    )

    parser.add_argument(
        "--name",
        help="Name of the state struct.",
        type=str,
        default="State",
    )

    parser.add_argument(
        "--mappings",
        help="Path to a json file containing additional type mappings.",
        type=str,
        default=None,
    )

    parser.add_argument(
        "--test",
        help="Run tests and exit.",
        action="store_true",
        default=False,
    )

    # allow 0 or more input files
    parser.add_argument("input", help="Input rust files to parse.", nargs="*", type=str)

    args = parser.parse_args()

    if args.test:
        return _tests()

    if not args.input:
        # no input files. maybe in stdin? try to read from stdin for 100ms
        timeout = 0.1
        import sys
        import select

        while sys.stdin in select.select([sys.stdin], [], [], timeout)[0]:
            line = sys.stdin.readline()
            if line:
                args.input.append(line.strip())
            else:
                break

    if not args.input:
        raise ValueError("No input files or stdin provided.")

    if args.verbose:
        for file in args.input:
            print(f"Processing file: {file}")

    all_structs = {}
    for file in args.input:
        with open(file, "r") as f:
            content = f.read()

        structs = parse_rust_structs(content)

        # check for duplicate struct names
        for name in structs:
            if name in all_structs:
                raise ValueError(f"Duplicate struct name: {name}")

        all_structs.update(structs)

    if args.verbose:
        print(f"Found structs: {all_structs.keys()}")

    env = all_structs.copy()

    if args.mappings:
        import json

        with open(args.mappings, "r") as f:
            mappings = json.load(f)

        if args.verbose:
            print(f"Loaded mappings: {mappings}")
        env.update(mappings)

    struct = env.get(args.name)

    if not struct:
        raise ValueError(f"Struct {name} not found in environment.")

    schema = struct_to_schema(struct, env=env)

    print(schema)


############################################
## MICROTEST
############################################


class _microtest_expect_error:
    def __init__(self, error: Exception, pattern: str | None = None):
        self.error = error
        self.pattern = pattern

    def __enter__(self):
        pass

    def __exit__(self, exc_type, exc_value, _traceback):
        if exc_type is None:
            raise AssertionError("Expected an error, but got none.")
        if exc_type != self.error:
            raise AssertionError(
                f"Expected an error of type {self.error}, but got {exc_type}."
            )
        if self.pattern:
            if self.pattern not in str(exc_value):
                raise AssertionError(
                    f"Expected an error containing '{self.pattern}', but got '{exc_value}'."
                )

        return True


def _microtest_rewrite_asserts(func, line_info=True):
    """Rewrite asserts in a function to if statements with more verbose error messages."""
    import ast
    import inspect
    import textwrap

    class RewriteAsserts(ast.NodeTransformer):
        def visit_Assert(self, node):
            message = ast.unparse(node.test)
            if node.msg:
                message += ". Message: " + ast.unparse(node.msg)
            file = inspect.getfile(func)
            lineno = node.lineno + inspect.getsourcelines(func)[1] - 1
            if line_info:
                message += f". At {file}:{lineno}"

            _raise = ast.Raise(
                exc=ast.Call(
                    func=ast.Name(id="AssertionError", ctx=ast.Load()),
                    args=[ast.Constant(value=f"Assertion failed: {message}")],
                    keywords=[],
                ),
                cause=None,
            )
            return ast.If(
                test=ast.UnaryOp(
                    op=ast.Not(),
                    operand=node.test,
                ),
                body=[_raise],
                orelse=[],
            )

    try:
        source = textwrap.dedent(inspect.getsource(func))
        tree = ast.parse(source)
        new_tree = RewriteAsserts().visit(tree)
        new_source = ast.unparse(new_tree)
        _namespace = globals().copy()
        exec(new_source, _namespace)
        return _namespace[func.__name__]
    except Exception as e:
        print(f"!! Failed to rewrite asserts in {func.__name__}: {e}")
        return func


def _microtest_run_tests():
    import inspect

    # frame hacking to get the locals and globals of the calling function
    _locals = dict(inspect.currentframe().f_back.f_locals)
    _globals = dict(inspect.currentframe().f_back.f_globals)
    _globals.update(_locals)

    tests = {
        name: value
        for name, value in _locals.items()
        if name.startswith("test_") and callable(value)
    }
    tests = {name: _microtest_rewrite_asserts(value) for name, value in tests.items()}
    (GREEN, RED, YELLOW, RESET) = ("\x1b[32m", "\x1b[31m", "\x1b[33m", "\x1b[0m")
    stats = {"passed": 0, "failed": 0, "skipped": 0}
    for name, test in tests.items():
        print(f"Test: {name}", end=" ")
        result = "failed"
        err = None
        try:
            # test()
            exec(test.__code__, _globals)
            result = "passed"
        except Exception as e:
            err = e
            result = "failed"

        if (
            err
            and isinstance(err, RuntimeError)
            and str(err).startswith("Test skipped")
        ):
            result = "skipped"

        if result == "failed":
            print(f"{RED}failed{RESET}: {err}")
            stats["failed"] += 1
        elif result == "skipped":
            print(f"{YELLOW}skipped{RESET}")
            stats["skipped"] += 1
        else:
            print(f"{GREEN}passed{RESET}")
            stats["passed"] += 1

    skipped_color = YELLOW if stats["skipped"] else GREEN
    failed_color = RED if stats["failed"] else GREEN
    print(
        f"Run {stats['passed'] + stats['failed']} tests: {GREEN}{stats['passed']} passed{RESET}, {skipped_color}{stats['skipped']} skipped{RESET}, {failed_color}{stats['failed']} failed{RESET}"
    )

    return stats["failed"]


def _microtest_skip_test(reason: str | None = None):
    message = "Test skipped" + (f": {reason}" if reason else "")
    raise RuntimeError(message)


############################################
## TESTS
############################################


def _tests():
    expect_error = _microtest_expect_error
    skip = _microtest_skip_test

    def test_array_parsing():
        value = [
            {"pub": "pub", "name": "enemy_mouse_n", "ptr": "", "type": "u32"},
            {"pub": "pub", "name": "enemy_mouse_arr", "ptr": "*mut", "type": "f32"},
        ]

        schema = struct_to_schema(value, env={})
        assert schema == "f*{enemy_mouse}"

    def test_array_parsing_with_different_names():
        value = [
            {"pub": "pub", "name": "enemy_mouse_n", "ptr": "", "type": "u32"},
            {"pub": "pub", "name": "enemy_mice_arr", "ptr": "*mut", "type": "f32"},
        ]

        with expect_error(ValueError, "enemy_mouse != enemy_mice"):
            struct_to_schema(value, env={})

    def test_array_parsing_with_different_types():
        value = [
            {"pub": "pub", "name": "enemy_mouse_n", "ptr": "", "type": "u32"},
            {"pub": "pub", "name": "enemy_mouse_arr", "ptr": "*mut", "type": "f64"},
        ]

        with expect_error(ValueError, "f64 not implemented"):
            struct_to_schema(value, env={})

    def test_skip_me():
        skip("This test is skipped.")

    exit(_microtest_run_tests())


############################################
## IF MAIN
############################################

if __name__ == "__main__":
    try:
        exit(main())
    except NotImplementedError as e:
        cprint(f"{e.__class__.__name__}: {e}", "blue")
        exit(1)
    except Exception as e:
        cprint(f"{e.__class__.__name__}: {e}", "red")
        exit(1)
