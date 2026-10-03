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
    ir.push_str("declare void @cleat_arith_fail()\n\n");
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
    ir.push_str(&format!(
        "define i32 @main() {{\n  call void @{mangle}()\n  ret i32 0\n}}\n"
    ));
    Ok(ir)
}

fn entry_mangle(entry: &str) -> String {
    let (package, ty) = entry.rsplit_once('.').unwrap_or(("demo", entry));
    crate::check::mangle(package, ty, "main")
}

fn emit_func(ir: &mut String, func: &crate::check::Func) {
    let ret = match func.ret {
        Ty::Int32 | Ty::NullableInt | Ty::Bool | Ty::Null => "i32",
        Ty::Unit | Ty::Void => "void",
    };
    let mut sig = Vec::new();
    for i in 0..func.params {
        sig.push(format!("i32 %p{i}"));
    }
    ir.push_str(&format!(
        "define {ret} @{}({}) {{\n",
        func.mangle,
        sig.join(", ")
    ));
    ir.push_str("entry:\n");
    for id in 0..func.local_count {
        ir.push_str(&format!("  %l{id} = alloca i32\n"));
    }
    for i in 0..func.params {
        ir.push_str(&format!("  store i32 %p{i}, ptr %l{i}\n"));
    }
    let mut n = 0u32;
    let terminated = emit_stmts(ir, &func.body, &mut n);
    if !terminated {
        if func.ret == Ty::Void || func.ret == Ty::Unit {
            ir.push_str("  ret void\n");
        } else {
            ir.push_str("  ret i32 0\n");
        }
    }
    ir.push_str("}\n\n");
}

fn emit_stmts(ir: &mut String, stmts: &[TStmt], n: &mut u32) -> bool {
    let mut terminated = false;
    for stmt in stmts {
        if terminated {
            break;
        }
        match stmt {
            TStmt::Local(id, init) => {
                if let Some(expr) = init {
                    let v = emit_expr(ir, expr, n);
                    ir.push_str(&format!("  store i32 {v}, ptr %l{id}\n"));
                }
            }
            TStmt::Assign(id, expr) => {
                let v = emit_expr(ir, expr, n);
                ir.push_str(&format!("  store i32 {v}, ptr %l{id}\n"));
            }
            TStmt::StoreGlobal(name, expr) => {
                let v = emit_expr(ir, expr, n);
                ir.push_str(&format!("  store i32 {v}, ptr @{name}\n"));
            }
            TStmt::Expr(expr) => {
                let _ = emit_expr(ir, expr, n);
            }
            TStmt::If(cond, then_body, else_body) => {
                let c = emit_cond(ir, cond, n);
                let id = fresh(n);
                ir.push_str(&format!(
                    "  br i1 {c}, label %then{id}, label %else{id}\nthen{id}:\n"
                ));
                let then_done = emit_stmts(ir, then_body, n);
                if !then_done {
                    ir.push_str(&format!("  br label %end{id}\n"));
                }
                ir.push_str(&format!("else{id}:\n"));
                let else_done = emit_stmts(ir, else_body, n);
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
                let c = emit_cond(ir, cond, n);
                ir.push_str(&format!(
                    "  br i1 {c}, label %body{id}, label %end{id}\nbody{id}:\n"
                ));
                let body_done = emit_stmts(ir, body, n);
                if !body_done {
                    ir.push_str(&format!("  br label %head{id}\n"));
                }
                ir.push_str(&format!("end{id}:\n"));
            }
            TStmt::Return(expr) => {
                if let Some(expr) = expr {
                    if matches!(expr, TExpr::Unit) {
                        ir.push_str("  ret void\n");
                    } else {
                        let v = emit_expr(ir, expr, n);
                        ir.push_str(&format!("  ret i32 {v}\n"));
                    }
                } else {
                    ir.push_str("  ret void\n");
                }
                terminated = true;
            }
        }
    }
    terminated
}

fn emit_cond(ir: &mut String, expr: &TExpr, n: &mut u32) -> String {
    let v = emit_expr(ir, expr, n);
    let id = fresh(n);
    ir.push_str(&format!("  %t{id} = icmp ne i32 {v}, 0\n"));
    format!("%t{id}")
}

fn emit_expr(ir: &mut String, expr: &TExpr, n: &mut u32) -> String {
    match expr {
        TExpr::Int(v) => format!("{v}"),
        TExpr::Bool(v) => {
            let bit = if *v { 1 } else { 0 };
            format!("{bit}")
        }
        TExpr::Local(id) => {
            let r = fresh(n);
            ir.push_str(&format!("  %t{r} = load i32, ptr %l{id}\n"));
            format!("%t{r}")
        }
        TExpr::Global(name) => {
            let r = fresh(n);
            ir.push_str(&format!("  %t{r} = load i32, ptr @{name}\n"));
            format!("%t{r}")
        }
        TExpr::Unit => "void".into(),
        TExpr::UnaryNeg(inner) => {
            let v = emit_expr(ir, inner, n);
            let id = fresh(n);
            ir.push_str(&format!(
                "  %t{id} = call {{i32, i1}} @llvm.ssub.with.overflow.i32(i32 0, i32 {v})\n"
            ));
            overflow(ir, id, n)
        }
        TExpr::Binary(op, left, right) => emit_bin(ir, *op, left, right, n),
        TExpr::Call(name, args, ret) => {
            let mut rendered = Vec::new();
            for arg in args {
                rendered.push(format!("i32 {}", emit_expr(ir, arg, n)));
            }
            let args = rendered.join(", ");
            if matches!(*ret, Ty::Void | Ty::Unit) {
                ir.push_str(&format!("  call void @{name}({args})\n"));
                "0".into()
            } else {
                let id = fresh(n);
                ir.push_str(&format!("  %t{id} = call i32 @{name}({args})\n"));
                format!("%t{id}")
            }
        }
    }
}

fn emit_bin(ir: &mut String, op: BinOp, left: &TExpr, right: &TExpr, n: &mut u32) -> String {
    let l = emit_expr(ir, left, n);
    let r = emit_expr(ir, right, n);
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
    let status = Command::new(clang)
        .arg("-fuse-ld=lld")
        .arg("-Wno-override-module")
        .arg(&ll)
        .arg(&runtime)
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
