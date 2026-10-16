//! How a program meets its host: foreign structs, the ways a program ends (section
//! 9.10), and the reports that do not reject a program.

use crate::common::{build_text, expect, run, scratch};
use std::path::PathBuf;
mod common;

fn write(dir: &std::path::Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

//@ rule: methods.foreign.result-written-void
//@ rule: methods.foreign.program-needs-passes
//@ rule: methods.foreign.call-uses-platform
//@ rule: methods.foreign.implementation-told-libraries
//@ rule: execution.program.calls-follow-rules
#[test]
fn foreign_structs() {
    let dir = scratch("foreign_structs");
    let c = write(
        &dir,
        "shapes.c",
        r#"
#include <stdint.h>
struct vec { double x; double y; };
struct seg { struct vec a; struct vec b; int32_t tag; };
struct vec vec_scale(struct vec v, double k) { struct vec r = { v.x * k, v.y * k }; return r; }
double seg_length2(struct seg s) { double dx = s.b.x - s.a.x, dy = s.b.y - s.a.y; return dx * dx + dy * dy + s.tag; }
struct seg seg_make(double x, double y, int32_t tag) { struct seg s = { { 0, 0 }, { x, y }, tag }; return s; }
void vec_negate_all(struct vec *v, int64_t n) { for (int64_t i = 0; i < n; i++) { v[i].x = -v[i].x; v[i].y = -v[i].y; } }
"#,
    );
    let source = write(
        &dir,
        "Structs.cleat",
        r#"
package t;

value class Vec {
    public Float64 x;
    public Float64 y;
}

value class Seg {
    public Vec a;
    public Vec b;
    public Int32 tag;
}

public class Structs {
    @Symbol("vec_scale")
    foreign static Vec scale(Vec v, Float64 k);

    @Symbol("seg_length2")
    foreign static Float64 length2(Seg s);

    @Symbol("seg_make")
    foreign static Seg make(Float64 x, Float64 y, Int32 tag);

    @Symbol("vec_negate_all")
    foreign static void negateAll(Vec[] items, Int count);

    public static void main() {
        Vec v = scale(new Vec(1.5, -2.0), 2.0);
        Console.println(v);
        Seg s = make(3.0, 4.0, 7);
        Console.println(s);
        Console.println(length2(s));
        // The function's writes reach the array, and no other holder of the value.
        Vec shared = new Vec(1.0, 2.0);
        Vec[] items = new Vec[] { shared, shared };
        negateAll(items, items.length());
        Console.println("" + items[0] + items[1] + shared);
    }
}
"#,
    );
    let exe = dir.join("structs.exe");
    cleatc::build_with(&[source.as_path()], "t.Structs", &exe, &[c])
        .unwrap_or_else(|e| panic!("{}", e[0].message));
    expect(
        &exe,
        &[],
        0,
        "Vec(3.0, -4.0)\nSeg(Vec(0.0, 0.0), Vec(3.0, 4.0), 7)\n32.0\nVec(-1.0, -2.0)Vec(-1.0, -2.0)Vec(1.0, 2.0)\n",
    );
}

/// Section 4.11: after the call, each array of value classes is constructed in the order
/// of the parameters, element by element, and the result after them. An element whose
/// compact constructor raises, and every element after it, keeps its value.
//@ rule: methods.foreign.arrays-constructed-order
#[test]
fn values_from_c_are_constructed_array_by_array_and_then_the_result() {
    let dir = scratch("foreign_constructed");
    let c = write(
        &dir,
        "divisions.c",
        r#"
#include <stdint.h>
struct division { int32_t quot; int32_t rem; };
struct division shift_all(struct division *a, int64_t n, struct division *b, int64_t m, int32_t rem) {
    for (int64_t k = 0; k < n; k++) a[k].quot += 10;
    for (int64_t k = 0; k < m; k++) { b[k].quot += 20; b[k].rem = rem; }
    struct division r = { 99, 1 };
    return r;
}
"#,
    );
    let source = write(
        &dir,
        "Divisions.cleat",
        r#"
package t;

class Trace {
    public static String log = "";

    public static String take() {
        var all = log;
        log = "";
        return all;
    }
}

value class Division {
    public Int32 quot;
    public Int32 rem;

    public Division {
        Trace.log = Trace.log + "d" + quot + " ";
        if (rem < 0) {
            throw new IllegalArgumentException("raised");
        }
    }
}

public class Divisions {
    @Symbol("shift_all")
    foreign static Division shiftAll(Division[] a, Int n, Division[] b, Int m, Int32 rem);

    public static void main() {
        Division[] a = new Division[] { new Division(1, 1), new Division(2, 1) };
        Division[] b = new Division[] { new Division(3, 1) };
        var skipped = Trace.take();
        Division r = shiftAll(a, a.length(), b, b.length(), 1);
        Console.println(Trace.take() + "| " + a[0].quot + " " + a[1].quot + " " + b[0].quot + " " + r.quot);
        Division[] c = new Division[] { new Division(1, 1) };
        Division[] d = new Division[] { new Division(2, 1), new Division(3, 1) };
        skipped = Trace.take();
        try {
            Division s = shiftAll(c, c.length(), d, d.length(), -1);
        } catch (IllegalArgumentException e) {
            Console.println(Trace.take() + e.message() + " | " + c[0].quot + " " + d[0].quot + " " + d[1].quot);
        }
    }
}
"#,
    );
    let exe = dir.join("divisions.exe");
    cleatc::build_with(&[source.as_path()], "t.Divisions", &exe, &[c])
        .unwrap_or_else(|e| panic!("{}", e[0].message));
    expect(
        &exe,
        &[],
        0,
        "d11 d12 d23 d99 | 11 12 23 99\nd11 d22 raised | 11 2 3\n",
    );
}

//@ rule: execution.startup.call-given-stack
//@ rule: omissions.implementation.exceeding-ends-program
//@ rule: execution.startup.running-out-stack
#[test]
fn a_full_stack_ends_the_program() {
    let (_, exe) = build_text(
        "stack",
        "t.Deep",
        "package t;\npublic class Deep {\n    static Int down(Int n) { return down(n + 1) + 1; }\n    public static void main() {\n        Console.println(\"start\");\n        Console.println(down(0));\n    }\n}\n",
    );
    let r = run(&exe, &[], false);
    assert_eq!(r.status, 1, "{}", r.err);
    assert_eq!(r.out, "start\n");
    assert!(r.err.contains("cannot be given stack space"), "{}", r.err);
}

//@ rule: execution.exceptions.exception-no-caller
//@ rule: execution.startup.exception-leaves-main
#[test]
fn an_uncaught_exception_ends_the_program_with_status_one() {
    let (_, exe) = build_text(
        "uncaught",
        "t.Fails",
        "package t;\npublic class Fails {\n    public static void main() {\n        Console.println(\"before\");\n        throw new IllegalStateException(\"gave up\");\n    }\n}\n",
    );
    let r = expect(&exe, &[], 1, "before\n");
    assert_eq!(r.err, "cleat.IllegalStateException: gave up\n");
}

//@ rule: execution.startup.process-exit-status
//@ rule: execution.startup.ends-program-once
#[test]
fn exit_ends_the_program_at_once() {
    let (_, exe) = build_text(
        "exit",
        "t.Leaves",
        "package t;\npublic class Leaves {\n    public static void main(String[] args) {\n        try {\n            Process.exit(args.length() + 40);\n        } finally {\n            Console.println(\"never\");\n        }\n    }\n}\n",
    );
    expect(&exe, &["a", "b"], 42, "");
    // A status outside 0 through 255 is refused, and the exception is reported.
    let (_, exe) = build_text(
        "exit_range",
        "t.Leaves",
        "package t;\npublic class Leaves {\n    public static void main() {\n        Process.exit(256);\n    }\n}\n",
    );
    let r = expect(&exe, &[], 1, "");
    assert!(
        r.err.starts_with("cleat.IllegalArgumentException"),
        "{}",
        r.err
    );
}

/// `args` holds what the host passed, in order and without the program's own name.
//@ rule: execution.startup.args-holds-arguments
//@ rule: execution.startup.no-arguments-length
#[test]
fn main_is_given_the_arguments_the_host_passed() {
    let (_, exe) = build_text(
        "arguments",
        "t.Echo",
        "package t;
public class Echo {
    public static void main(String[] args) {
        Console.println(args.length());
        for (var i = 0; i < args.length(); i++) {
            Console.println(\"[\" + args[i] + \"]\");
        }
    }
}
",
    );
    expect(
        &exe,
        &["first", "second word", "3"],
        0,
        "3
[first]
[second word]
[3]
",
    );
    // With no arguments the array has length zero.
    expect(
        &exe,
        &[],
        0,
        "0
",
    );
}

//@ rule: execution.startup.implementation-told-entry
#[test]
fn main_may_take_no_arguments_and_must_be_one() {
    let dir = scratch("entry");
    let both = write(
        &dir,
        "Both.cleat",
        "package t;\npublic class Both {\n    public static void main() { }\n    public static void main(String[] args) { }\n}\n",
    );
    let err = cleatc::build_roots(&[both.as_path()], "t.Both", &dir.join("both.exe"))
        .expect_err("two mains");
    assert!(
        err[0].message.contains("not an entry class"),
        "{}",
        err[0].message
    );
    let none = write(
        &dir,
        "Nothing.cleat",
        "package t;\npublic class Nothing {\n    static void main() { }\n}\n",
    );
    let err = cleatc::build_roots(&[none.as_path()], "t.Nothing", &dir.join("none.exe"))
        .expect_err("a private main");
    assert!(
        err[0].message.contains("not an entry class"),
        "{}",
        err[0].message
    );
}

#[test]
fn a_use_of_a_deprecated_declaration_is_reported_and_not_rejected() {
    let dir = scratch("deprecated");
    let old = write(
        &dir,
        "Old.cleat",
        "package t;\npublic class Old {\n    @Deprecated(\"use fresh\")\n    public static Int stale() { return 1; }\n    public static Int own() { return stale(); }\n}\n",
    );
    let user = write(
        &dir,
        "User.cleat",
        "package t;\npublic class User {\n    public static void main() {\n        Console.println(Old.stale() + Old.own());\n    }\n}\n",
    );
    let exe = dir.join("user.exe");
    let warnings = cleatc::build_roots(&[old.as_path(), user.as_path()], "t.User", &exe)
        .unwrap_or_else(|e| panic!("{}", e[0].message));
    // The use in another file is reported, and the use in the declaring file is not.
    assert_eq!(
        warnings.len(),
        1,
        "{:?}",
        warnings.iter().map(|w| &w.message).collect::<Vec<_>>()
    );
    assert!(
        warnings[0].file.ends_with("User.cleat") && warnings[0].message.contains("use fresh"),
        "{}",
        warnings[0].message
    );
    expect(&exe, &[], 0, "2\n");
}
