//! Every method of the prelude whose body the implementation supplies, and why it is
//! not written in the language. A method that can be written over these is source. The
//! test fails when the prelude gains an `@Intrinsic` that is not listed here, or keeps
//! one that the list has dropped.

use std::collections::BTreeMap;
use std::path::Path;

/// A class, why its listed methods are intrinsic, and their signatures.
const INTRINSICS: &[(&str, &str, &[&str])] = &[
    (
        "Int",
        "The machine's 64-bit arithmetic, comparison and bit operations. A narrower integer is taken as it is; a UInt64 and a float are checked.",
        &[
            "plus(Int)", "minus(Int)", "times(Int)", "lessThan(Int)", "truncatingDiv(Int)", "truncatingRem(Int)", "wrappingPlus(Int)", "wrappingMinus(Int)", "wrappingTimes(Int)", "and(Int)", "or(Int)",
            "xor(Int)", "shiftLeft(Int)", "shiftRight(Int)", "from(Int8)", "from(Int16)", "from(Int32)", "from(UInt8)", "from(UInt16)", "from(UInt32)", "from(UInt64)", "from(Float64)",
        ],
    ),
    (
        "UInt64",
        "The machine's unsigned 64-bit arithmetic, comparison and bit operations, and the conversions from the classes an Int cannot stand for.",
        &[
            "plus(UInt64)", "minus(UInt64)", "times(UInt64)", "lessThan(UInt64)", "truncatingDiv(UInt64)", "truncatingRem(UInt64)", "wrappingPlus(UInt64)", "wrappingMinus(UInt64)", "wrappingTimes(UInt64)",
            "and(UInt64)", "or(UInt64)", "xor(UInt64)", "shiftLeft(Int)", "shiftRight(Int)", "from(Int)", "from(Float64)",
        ],
    ),
    ("Int8", "The two conversions from an Int: the one that checks and the one that keeps the low bits. All its arithmetic is done in Int.", &["wrapping(Int)", "from(Int)"]),
    ("Int16", "As Int8.", &["wrapping(Int)", "from(Int)"]),
    ("Int32", "As Int8.", &["wrapping(Int)", "from(Int)"]),
    ("UInt8", "As Int8.", &["wrapping(Int)", "from(Int)"]),
    ("UInt16", "As Int8.", &["wrapping(Int)", "from(Int)"]),
    ("UInt32", "As Int8.", &["wrapping(Int)", "from(Int)"]),
    (
        "Float64",
        "The operations of IEEE 754, each one instruction or one library routine. Printing and reading decimal digits are whole algorithms, still in the runtime. An Int and a UInt64 convert in one step, because two steps would round twice.",
        &[
            "plus(Float64)", "minus(Float64)", "times(Float64)", "div(Float64)", "negate()", "lessThan(Float64)", "atMost(Float64)", "totalOrder(Float64)", "floor()", "ceil()", "truncate()", "round()",
            "sqrt()", "abs()", "toFixed(Int)", "parse(String)", "from(Int)", "from(UInt64)", "from(Float32)", "nearest(Int)", "nearest(UInt64)", "nearest(Float32)",
        ],
    ),
    (
        "Float32",
        "As Float64.",
        &[
            "plus(Float32)", "minus(Float32)", "times(Float32)", "div(Float32)", "negate()", "lessThan(Float32)", "atMost(Float32)", "totalOrder(Float32)", "floor()", "ceil()", "truncate()", "round()",
            "sqrt()", "abs()", "toFixed(Int)", "parse(String)", "from(Int)", "from(UInt64)", "from(Float64)", "nearest(Int)", "nearest(UInt64)", "nearest(Float64)",
        ],
    ),
    ("Char", "A scalar and its number, and the tables of Unicode.", &["from(Int)", "code()", "isLetter()", "isDigit()", "isWhitespace()", "toLowerCase()", "toUpperCase()"]),
    (
        "String",
        "The storage of a string: its length, one scalar, a part, two strings joined, and a string made from scalars. The full case mapping needs the tables of Unicode.",
        &["length()", "get(Int)", "substring(Int, Int)", "join(String)", "fromChars(Char[], Int)", "toLowerCase()", "toUpperCase()"],
    ),
    ("Array", "The storage of an array, which no class can declare.", &["Array()", "length()", "get(Int)", "set(Int, T)", "unfilled(Int)"]),
    (
        "Object",
        "Identity and the class of an object, and the defaults that read the fields of any class.",
        &["equals(Object, Object)", "hashCode(Object)", "toString(Object)", "identical(Object, Object)", "getClass()"],
    ),
    (
        "Class",
        "The description of a class that the compiler writes into the program. The mirrors `Field`, `Method` and `Parameter` are source over these.",
        &[
            "Class()", "getName()", "getSuperclass()", "getInterfaces()", "getAnnotation()", "annotatedFields()", "annotatedMethods()", "fieldName(Int)", "fieldIsStatic(Int)", "fieldAnnotation(Int)",
            "fieldGet(Int, Object)", "fieldSet(Int, Object, Object)", "methodName(Int)", "methodIsStatic(Int)", "parameterCount(Int)", "parameterName(Int, Int)", "methodAnnotation(Int)",
            "parameterAnnotation(Int, Int)", "methodInvoke(Int, Object, Object[])",
        ],
    ),
    ("Pointer", "An address of the host.", &["zero()"]),
    ("Atomic", "One step of the machine on a cell that holds a reference, which the collector must see.", &["get()", "set(T)", "compareAndSet(T, T)"]),
    ("AtomicInt", "One step of the machine on a cell.", &["get()", "set(Int)", "compareAndSet(Int, Int)"]),
    ("Thread", "Threads of the host.", &["isAlive()", "create()", "launch()", "finished(Int)", "running()", "adopt(Thread)", "pause(Int)"]),
    ("Lock", "A lock of the host.", &["create()", "lock()", "unlock()", "held()", "park(Pointer, Int)"]),
    ("Condition", "A queue of waiting threads of the host.", &["create()", "wake(Boolean)"]),
    ("Console", "The streams of the host.", &["readLine()", "write(Int, String)"]),
    ("File", "The files of the host.", &["readBytes(String)", "writeBytes(String, UInt8[])"]),
    ("Process", "The end of the program.", &["halt(Int)"]),
];

/// The intrinsic methods each file of the prelude declares, as `name(ParameterTypes)`.
fn declared() -> BTreeMap<String, Vec<String>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("prelude");
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("cleat") {
            continue;
        }
        let class = path.file_stem().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
        let mut found = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if *line != "@Intrinsic" {
                continue;
            }
            // The declaration follows, after any other annotation.
            let decl = lines[i + 1..].iter().find(|l| !l.starts_with('@')).unwrap();
            let open = decl.find('(').unwrap();
            let name = decl[..open].rsplit(' ').next().unwrap();
            let params: Vec<String> = decl[open + 1..decl.rfind(')').unwrap()]
                .split(',')
                .map(|p| p.trim().replace("@Nullable ", ""))
                .filter(|p| !p.is_empty())
                .map(|p| p.rsplit_once(' ').map(|(ty, _)| ty.to_string()).unwrap_or(p))
                .collect();
            found.push(format!("{name}({})", params.join(", ")));
        }
        if !found.is_empty() {
            out.insert(class, found);
        }
    }
    out
}

#[test]
fn every_intrinsic_of_the_prelude_is_listed_with_its_reason() {
    let declared = declared();
    let mut wrong = Vec::new();
    for (class, _, listed) in INTRINSICS {
        let have = declared.get(*class).cloned().unwrap_or_default();
        for m in *listed {
            if !have.iter().any(|h| h == m) {
                wrong.push(format!("`{class}.{m}` is listed and is not intrinsic in the prelude: drop it from the list"));
            }
        }
        for h in &have {
            if !listed.contains(&h.as_str()) {
                wrong.push(format!("`{class}.{h}` is intrinsic and is not listed: write it in the language, or say here why it cannot be"));
            }
        }
    }
    for class in declared.keys() {
        if !INTRINSICS.iter().any(|(c, _, _)| c == class) {
            wrong.push(format!("`{class}` declares intrinsic methods and is not listed"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    let total: usize = INTRINSICS.iter().map(|(_, _, m)| m.len()).sum();
    println!("{total} methods of the prelude are intrinsic, in {} classes", INTRINSICS.len());
    for (class, why, methods) in INTRINSICS {
        println!("  {class}: {}. {why}", methods.len());
    }
}

/// The classes that were bodiless and are now written in the language stay that way.
#[test]
fn the_classes_written_in_the_language_declare_no_intrinsic() {
    let declared = declared();
    for class in ["Boolean", "Null", "List", "Map", "StringBuilder", "Field", "Method", "Parameter", "Scope", "Task", "Rational", "BigInt", "Floats"] {
        assert!(!declared.contains_key(class), "`{class}` declares an intrinsic method");
    }
}
