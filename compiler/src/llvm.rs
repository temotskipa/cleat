use crate::ast::BinOp;
use crate::check::{Checked, TExpr, TStmt, Ty};
use std::path::Path;
use std::process::Command;

pub fn emit(project: &Checked, entry: &str) -> Result<String, Vec<crate::check::Diagnostic>> {
    let mangle = entry_mangle(entry);
    if !project.funcs.iter().any(|f| f.mangle == mangle) {
        return Err(vec![crate::check::Diagnostic {
            file: Path::new(".").to_path_buf(),
            line: 1,
            column: 1,
            message: format!("entry type {entry} has no static main"),
        }]);
    }
    let mut ir = String::from("target triple = \"x86_64-pc-windows-msvc\"\n\n");
    ir.push_str("declare void @cleat_arith_fail()\n");
    ir.push_str("declare void @cleat_push_root(ptr)\n");
    ir.push_str("declare void @cleat_pop_root()\n");
    ir.push_str("declare void @cleat_set_parent(i32, i32)\n");
    ir.push_str("declare ptr @cleat_alloc(i32, i32, i32)\n");
    ir.push_str("declare ptr @cleat_alloc_array(i32, i32)\n");
    ir.push_str("declare ptr @cleat_box_i32(i32)\n");
    ir.push_str("declare ptr @cleat_get_class(ptr)\n");
    ir.push_str("declare i32 @cleat_is_instance(ptr, ptr)\n");
    ir.push_str("declare i32 @cleat_has(ptr, ptr)\n");
    ir.push_str("declare ptr @cleat_array_load(ptr, i32)\n");
    ir.push_str("declare void @cleat_array_store(ptr, i32, ptr)\n\n");
    for global in &project.globals {
        ir.push_str(&format!(
            "@{} = global i32 {}\n",
            global.mangle, global.init
        ));
    }
    if !project.globals.is_empty() {
        ir.push('\n');
    }
    for func in &project.funcs {
        emit_func(&mut ir, func);
    }
    let mut startup = String::new();
    for layout in &project.layouts {
        startup.push_str(&format!(
            "  call void @cleat_set_parent(i32 {}, i32 {})\n",
            layout.id, layout.parent
        ));
    }
    ir.push_str(&format!(
        "define i32 @main() {{\n{startup}  call void @{mangle}()\n  ret i32 0\n}}\n"
    ));
    Ok(ir)
}

fn entry_mangle(entry: &str) -> String {
    let (package, ty) = entry.rsplit_once('.').unwrap_or(("demo", entry));
    crate::check::mangle(package, ty, "main")
}

fn emit_func(ir: &mut String, func: &crate::check::Func) {
    let ret = llvm_ty(func.ret);
    let mut sig = Vec::new();
    for i in 0..func.params {
        let kind = if func.local_is_ref.get(i).copied().unwrap_or(false) {
            "ptr"
        } else {
            "i32"
        };
        sig.push(format!("{kind} %p{i}"));
    }
    ir.push_str(&format!(
        "define {ret} @{}({}) {{\n",
        func.mangle,
        sig.join(", ")
    ));
    ir.push_str("entry:\n");
    for id in 0..func.local_count {
        let kind = if func.local_is_ref.get(id).copied().unwrap_or(false) {
            "ptr"
        } else {
            "i32"
        };
        ir.push_str(&format!("  %l{id} = alloca {kind}\n"));
    }
    let ref_locals: Vec<usize> = (0..func.local_count)
        .filter(|id| func.local_is_ref.get(*id).copied().unwrap_or(false))
        .collect();
    for id in &ref_locals {
        ir.push_str(&format!("  call void @cleat_push_root(ptr %l{id})\n"));
    }
    for i in 0..func.params {
        let kind = if func.local_is_ref.get(i).copied().unwrap_or(false) {
            "ptr"
        } else {
            "i32"
        };
        ir.push_str(&format!("  store {kind} %p{i}, ptr %l{i}\n"));
    }
    let mut n = 0u32;
    let terminated = emit_stmts(ir, &func.body, &func.local_is_ref, ref_locals.len(), &mut n);
    if !terminated {
        pop_roots(ir, ref_locals.len());
        if func.ret == Ty::Void || func.ret == Ty::Unit {
            ir.push_str("  ret void\n");
        } else if func.ret.is_ref() {
            ir.push_str("  ret ptr null\n");
        } else {
            ir.push_str("  ret i32 0\n");
        }
    }
    ir.push_str("}\n\n");
}

fn llvm_ty(ty: Ty) -> &'static str {
    if ty.is_ref() {
        "ptr"
    } else if matches!(ty, Ty::Unit | Ty::Void) {
        "void"
    } else {
        "i32"
    }
}

fn pop_roots(ir: &mut String, count: usize) {
    for _ in 0..count {
        ir.push_str("  call void @cleat_pop_root()\n");
    }
}

fn emit_stmts(
    ir: &mut String,
    stmts: &[TStmt],
    local_is_ref: &[bool],
    roots: usize,
    n: &mut u32,
) -> bool {
    let mut terminated = false;
    for stmt in stmts {
        if terminated {
            break;
        }
        match stmt {
            TStmt::Local(id, init) => {
                if let Some(expr) = init {
                    let v = emit_expr(ir, expr, local_is_ref, n);
                    let kind = if local_is_ref.get(*id).copied().unwrap_or(false) {
                        "ptr"
                    } else {
                        "i32"
                    };
                    ir.push_str(&format!("  store {kind} {v}, ptr %l{id}\n"));
                }
            }
            TStmt::Assign(id, expr) => {
                let v = emit_expr(ir, expr, local_is_ref, n);
                let kind = if local_is_ref.get(*id).copied().unwrap_or(false) {
                    "ptr"
                } else {
                    "i32"
                };
                ir.push_str(&format!("  store {kind} {v}, ptr %l{id}\n"));
            }
            TStmt::StoreGlobal(name, expr) => {
                let v = emit_expr(ir, expr, local_is_ref, n);
                ir.push_str(&format!("  store i32 {v}, ptr @{name}\n"));
            }
            TStmt::FieldStore { base, offset, is_ref, value } => {
                let obj = emit_expr(ir, base, local_is_ref, n);
                let stored = emit_expr(ir, value, local_is_ref, n);
                let slot = fresh(n);
                let addr = 16 + offset;
                ir.push_str(&format!("  %t{slot} = getelementptr i8, ptr {obj}, i32 {addr}\n"));
                let kind = if *is_ref { "ptr" } else { "i32" };
                ir.push_str(&format!("  store {kind} {stored}, ptr %t{slot}\n"));
            }
            TStmt::IndexStore { base, index, value } => {
                let arr = emit_expr(ir, base, local_is_ref, n);
                let idx = emit_expr(ir, index, local_is_ref, n);
                let stored = emit_expr(ir, value, local_is_ref, n);
                ir.push_str(&format!("  call void @cleat_array_store(ptr {arr}, i32 {idx}, ptr {stored})\n"));
            }
            TStmt::Expr(expr) => {
                let _ = emit_expr(ir, expr, local_is_ref, n);
            }
            TStmt::If(cond, then_body, else_body) => {
                let c = emit_cond(ir, cond, local_is_ref, n);
                let id = fresh(n);
                ir.push_str(&format!(
                    "  br i1 {c}, label %then{id}, label %else{id}\nthen{id}:\n"
                ));
                let then_done = emit_stmts(ir, then_body, local_is_ref, roots, n);
                if !then_done {
                    ir.push_str(&format!("  br label %end{id}\n"));
                }
                ir.push_str(&format!("else{id}:\n"));
                let else_done = emit_stmts(ir, else_body, local_is_ref, roots, n);
                if !else_done {
                    ir.push_str(&format!("  br label %end{id}\n"));
                }
                if then_done && else_done {
                    terminated = true;
                } else {
                    ir.push_str(&format!("end{id}:\n"));
                }
            }
            TStmt::While(cond, body) => {
                let id = fresh(n);
                ir.push_str(&format!("  br label %head{id}\nhead{id}:\n"));
                let c = emit_cond(ir, cond, local_is_ref, n);
                ir.push_str(&format!(
                    "  br i1 {c}, label %body{id}, label %end{id}\nbody{id}:\n"
                ));
                let body_done = emit_stmts(ir, body, local_is_ref, roots, n);
                if !body_done {
                    ir.push_str(&format!("  br label %head{id}\n"));
                }
                ir.push_str(&format!("end{id}:\n"));
            }
            TStmt::Return(expr) => {
                if let Some(expr) = expr {
                    if matches!(expr, TExpr::Unit) {
                        pop_roots(ir, roots);
                        ir.push_str("  ret void\n");
                    } else {
                        let v = emit_expr(ir, expr, local_is_ref, n);
                        pop_roots(ir, roots);
                        let kind = value_kind(expr, local_is_ref);
                        ir.push_str(&format!("  ret {kind} {v}\n"));
                    }
                } else {
                    pop_roots(ir, roots);
                    ir.push_str("  ret void\n");
                }
                terminated = true;
            }
        }
    }
    terminated
}

fn value_kind(expr: &TExpr, local_is_ref: &[bool]) -> &'static str {
    match expr {
        TExpr::Local(id) if local_is_ref.get(*id).copied().unwrap_or(false) => "ptr",
        TExpr::Alloc { .. }
        | TExpr::AllocArray { .. }
        | TExpr::BoxI32(_)
        | TExpr::GetClass(_)
        | TExpr::Index { .. } => "ptr",
        TExpr::Field { is_ref: true, .. } => "ptr",
        _ => "i32",
    }
}

fn emit_cond(ir: &mut String, expr: &TExpr, local_is_ref: &[bool], n: &mut u32) -> String {
    let v = emit_expr(ir, expr, local_is_ref, n);
    let id = fresh(n);
    ir.push_str(&format!("  %t{id} = icmp ne i32 {v}, 0\n"));
    format!("%t{id}")
}

fn emit_expr(ir: &mut String, expr: &TExpr, local_is_ref: &[bool], n: &mut u32) -> String {
    match expr {
        TExpr::Int(v) => format!("{v}"),
        TExpr::Bool(v) => {
            let bit = if *v { 1 } else { 0 };
            format!("{bit}")
        }
        TExpr::Local(id) => {
            let r = fresh(n);
            let kind = if local_is_ref.get(*id).copied().unwrap_or(false) {
                "ptr"
            } else {
                "i32"
            };
            ir.push_str(&format!("  %t{r} = load {kind}, ptr %l{id}\n"));
            format!("%t{r}")
        }
        TExpr::Global(name) => {
            let r = fresh(n);
            ir.push_str(&format!("  %t{r} = load i32, ptr @{name}\n"));
            format!("%t{r}")
        }
        TExpr::Unit => "void".into(),
        TExpr::UnaryNeg(inner) => {
            let v = emit_expr(ir, inner, local_is_ref, n);
            let id = fresh(n);
            ir.push_str(&format!(
                "  %t{id} = call {{i32, i1}} @llvm.ssub.with.overflow.i32(i32 0, i32 {v})\n"
            ));
            overflow(ir, id, n)
        }
        TExpr::Binary(op, left, right) => emit_bin(ir, *op, left, right, local_is_ref, n),
        TExpr::Call(name, args, ret) => {
            let mut rendered = Vec::new();
            for arg in args {
                let value = emit_expr(ir, arg, local_is_ref, n);
                rendered.push(format!("{} {value}", value_kind(arg, local_is_ref)));
            }
            let args = rendered.join(", ");
            if matches!(*ret, Ty::Void | Ty::Unit) {
                ir.push_str(&format!("  call void @{name}({args})\n"));
                "0".into()
            } else {
                let id = fresh(n);
                let kind = llvm_ty(*ret);
                ir.push_str(&format!("  %t{id} = call {kind} @{name}({args})\n"));
                format!("%t{id}")
            }
        }
        TExpr::Alloc { class, nrefs, payload } => {
            let id = fresh(n);
            ir.push_str(&format!(
                "  %t{id} = call ptr @cleat_alloc(i32 {payload}, i32 {class}, i32 {nrefs})\n"
            ));
            format!("%t{id}")
        }
        TExpr::AllocArray { len, elem } => {
            let length = emit_expr(ir, len, local_is_ref, n);
            let id = fresh(n);
            let elem_id = if *elem == u32::MAX { 0 } else { *elem };
            ir.push_str(&format!(
                "  %t{id} = call ptr @cleat_alloc_array(i32 {length}, i32 {elem_id})\n"
            ));
            format!("%t{id}")
        }
        TExpr::Field { base, offset, is_ref } => {
            let obj = emit_expr(ir, base, local_is_ref, n);
            let slot = fresh(n);
            let addr = 16 + offset;
            ir.push_str(&format!("  %t{slot} = getelementptr i8, ptr {obj}, i32 {addr}\n"));
            let loaded = fresh(n);
            let kind = if *is_ref { "ptr" } else { "i32" };
            ir.push_str(&format!("  %t{loaded} = load {kind}, ptr %t{slot}\n"));
            format!("%t{loaded}")
        }
        TExpr::Index { base, index } => {
            let arr = emit_expr(ir, base, local_is_ref, n);
            let idx = emit_expr(ir, index, local_is_ref, n);
            let id = fresh(n);
            ir.push_str(&format!("  %t{id} = call ptr @cleat_array_load(ptr {arr}, i32 {idx})\n"));
            format!("%t{id}")
        }
        TExpr::BoxI32(inner) => {
            let value = emit_expr(ir, inner, local_is_ref, n);
            let id = fresh(n);
            ir.push_str(&format!("  %t{id} = call ptr @cleat_box_i32(i32 {value})\n"));
            format!("%t{id}")
        }
        TExpr::GetClass(inner) => {
            let value = emit_expr(ir, inner, local_is_ref, n);
            let id = fresh(n);
            ir.push_str(&format!("  %t{id} = call ptr @cleat_get_class(ptr {value})\n"));
            format!("%t{id}")
        }
        TExpr::IsInstance(class_obj, obj) => {
            let left = emit_expr(ir, class_obj, local_is_ref, n);
            let right = emit_expr(ir, obj, local_is_ref, n);
            let id = fresh(n);
            ir.push_str(&format!(
                "  %t{id} = call i32 @cleat_is_instance(ptr {left}, ptr {right})\n"
            ));
            format!("%t{id}")
        }
        TExpr::Has(class_obj, ann) => {
            let left = emit_expr(ir, class_obj, local_is_ref, n);
            let right = emit_expr(ir, ann, local_is_ref, n);
            let id = fresh(n);
            ir.push_str(&format!("  %t{id} = call i32 @cleat_has(ptr {left}, ptr {right})\n"));
            format!("%t{id}")
        }
        TExpr::RefEq(left, right) | TExpr::RefNe(left, right) => {
            let l = emit_expr(ir, left, local_is_ref, n);
            let r = emit_expr(ir, right, local_is_ref, n);
            let id = fresh(n);
            let pred = if matches!(expr, TExpr::RefEq(_, _)) {
                "eq"
            } else {
                "ne"
            };
            ir.push_str(&format!("  %c{id} = icmp {pred} ptr {l}, {r}\n"));
            ir.push_str(&format!("  %t{id} = zext i1 %c{id} to i32\n"));
            format!("%t{id}")
        }
    }
}

fn emit_bin(ir: &mut String, op: BinOp, left: &TExpr, right: &TExpr, local_is_ref: &[bool], n: &mut u32) -> String {
    let l = emit_expr(ir, left, local_is_ref, n);
    let r = emit_expr(ir, right, local_is_ref, n);
    let id = fresh(n);
    match op {
        BinOp::Add => {
            ir.push_str(&format!(
                "  %t{id} = call {{i32, i1}} @llvm.sadd.with.overflow.i32(i32 {l}, i32 {r})\n"
            ));
            overflow(ir, id, n)
        }
        BinOp::Sub => {
            ir.push_str(&format!(
                "  %t{id} = call {{i32, i1}} @llvm.ssub.with.overflow.i32(i32 {l}, i32 {r})\n"
            ));
            overflow(ir, id, n)
        }
        BinOp::Mul => {
            ir.push_str(&format!(
                "  %t{id} = call {{i32, i1}} @llvm.smul.with.overflow.i32(i32 {l}, i32 {r})\n"
            ));
            overflow(ir, id, n)
        }
        BinOp::Div => check_div(ir, &l, &r, true, n),
        BinOp::Rem => check_div(ir, &l, &r, false, n),
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
            let pred = match op {
                BinOp::Eq => "eq",
                BinOp::Ne => "ne",
                BinOp::Lt => "slt",
                BinOp::Gt => "sgt",
                BinOp::Le => "sle",
                BinOp::Ge => "sge",
                _ => unreachable!(),
            };
            ir.push_str(&format!("  %c{id} = icmp {pred} i32 {l}, {r}\n"));
            ir.push_str(&format!("  %t{id} = zext i1 %c{id} to i32\n"));
            format!("%t{id}")
        }
    }
}

fn overflow(ir: &mut String, id: u32, n: &mut u32) -> String {
    let flag = fresh(n);
    let val = fresh(n);
    ir.push_str(&format!("  %t{val} = extractvalue {{i32, i1}} %t{id}, 0\n"));
    ir.push_str(&format!(
        "  %t{flag} = extractvalue {{i32, i1}} %t{id}, 1\n"
    ));
    let cont = fresh(n);
    ir.push_str(&format!(
        "  br i1 %t{flag}, label %ov{cont}, label %ok{cont}\nov{cont}:\n  call void @cleat_arith_fail()\n  unreachable\nok{cont}:\n"
    ));
    format!("%t{val}")
}

fn check_div(ir: &mut String, l: &str, r: &str, exact: bool, n: &mut u32) -> String {
    let z = fresh(n);
    ir.push_str(&format!("  %t{z} = icmp eq i32 {r}, 0\n"));
    let min = fresh(n);
    ir.push_str(&format!("  %t{min} = icmp eq i32 {l}, -2147483648\n"));
    let neg = fresh(n);
    ir.push_str(&format!("  %t{neg} = icmp eq i32 {r}, -1\n"));
    let both = fresh(n);
    ir.push_str(&format!("  %t{both} = and i1 %t{min}, %t{neg}\n"));
    let bad = fresh(n);
    ir.push_str(&format!("  %t{bad} = or i1 %t{z}, %t{both}\n"));
    let id = fresh(n);
    ir.push_str(&format!(
        "  br i1 %t{bad}, label %divbad{id}, label %divok{id}\ndivbad{id}:\n  call void @cleat_arith_fail()\n  unreachable\ndivok{id}:\n"
    ));
    let q = fresh(n);
    ir.push_str(&format!("  %t{q} = sdiv i32 {l}, {r}\n"));
    if !exact {
        let rem = fresh(n);
        ir.push_str(&format!("  %t{rem} = srem i32 {l}, {r}\n"));
        return format!("%t{rem}");
    }
    let prod = fresh(n);
    ir.push_str(&format!("  %t{prod} = mul i32 %t{q}, {r}\n"));
    let same = fresh(n);
    ir.push_str(&format!("  %t{same} = icmp eq i32 %t{prod}, {l}\n"));
    let next = fresh(n);
    ir.push_str(&format!(
        "  br i1 %t{same}, label %divexact{next}, label %divinexact{next}\ndivinexact{next}:\n  call void @cleat_arith_fail()\n  unreachable\ndivexact{next}:\n"
    ));
    format!("%t{q}")
}

fn fresh(n: &mut u32) -> u32 {
    let id = *n;
    *n += 1;
    id
}

pub fn link(ir: &str, output: &Path) -> Result<(), String> {
    let dir = output.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let ll = output.with_extension("ll");
    std::fs::write(&ll, ir).map_err(|err| err.to_string())?;
    let clang = std::env::var("CLEAT_CLANG")
        .unwrap_or_else(|_| r"C:\Program Files\LLVM\bin\clang.exe".to_string());
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/arith.c");
    let heap = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/heap.c");
    let status = Command::new(clang)
        .arg("-fuse-ld=lld")
        .arg("-Wno-override-module")
        .arg(&ll)
        .arg(&runtime)
        .arg(&heap)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|err| format!("cannot run clang: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("clang exited with {status}"))
    }
}
