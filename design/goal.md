Finish the Cleat revision in C:\Users\ttski\Projects\cleat: one consistent specification, a prelude written in Cleat, and a compiler that implements both.

Read first: design/foundations.md (the decision record; rows marked "Decided" are the owner's rulings and are not reopened), README.md (chapter status), the revised chapters in spec/, and design/programs/.

Work in this order. Finish and verify each step before starting the next.

1. Spec. Apply the two newest rulings: use-site wildcards return to chapter 7 beside `in` and `out`, and a program may declare its own annotations as in Java (elements, `@Target`, read at run time) in chapter 8. Then revise the pending chapters in place: 12 flow, 13 concurrency, 1 source, 3 visibility, 10 omissions, and last 11, a grammar that derives every example in the spec. Write as the revised chapters do: plain rules, an example for each, no rule stated twice.
2. Prelude. Write every prelude type the spec names as .cleat source under prelude/, using only what a user class may use, plus foreign or intrinsic method bodies. Anything that needs more becomes a language rule or goes on the list of admitted exceptions in foundations.md.
3. Tests from the spec. Turn each fenced example in spec/ into a test, wrapped where it is a fragment, that `cleatc check` must accept, or must reject where the text says "rejected". The spec and its tests change together.
4. Compiler (compiler/, Rust, emits LLVM IR for clang). Bring it from the Int32 subset to the whole specification, in slices that each end with passing tests: numbers and literals; classes, value classes, interfaces, enums; null and narrowing; reified generics with variance and wildcards; lambdas, switch, exceptions, using; annotations and mirrors; strings, arrays, collections; threads, locks, atomics; foreign calls. Keep the collector precise.
5. Programs. Each file in design/programs/ builds and runs with the output its header comment promises. Add one program for wildcards and one for a program-declared annotation read at run time.

Done when all of these hold, each shown by command output in the session:
- README lists every chapter as Revised, and no chapter contradicts another chapter or foundations.md.
- `cargo test` in compiler/ passes with no ignored tests, and the suite includes the spec-example tests and one run test for each program in design/programs/.
- `cleatc check` succeeds on the prelude.
- A final report lists every call made without the owner. Each one is recorded in foundations.md as "Assumed".

Rules.
- Do not ask before an ordinary design call. Choose, record it under "Calls made while rewriting" in foundations.md, and continue. Stop and ask only when a ruling marked Decided proves unworkable; say why and give the alternatives.
- Never report that something works without running it. Report a failing or skipped check as failing or skipped.
- When the spec is wrong or cannot be implemented, change the spec and its tests first, then the compiler. The compiler must not quietly disagree with the text.
- Commit after each verified step, on a branch named `revision`. Commit the collector work that was already uncommitted first, on its own. Never push and never rewrite history.
- clang is at C:\Program Files\LLVM\bin\clang.exe, or where CLEAT_CLANG points. If a tool is missing, say so. Do not weaken a test to get around it.
- The language name is undecided. Do not rename anything.
