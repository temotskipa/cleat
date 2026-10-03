use crate::ast::{Audience, BinOp, Block, Expr, ExprKind, Field, Method, ResultType, Stmt, Unit};
use crate::parse::line_col;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct Checked {
    pub funcs: Vec<Func>,
    pub globals: Vec<Global>,
}

#[derive(Clone, Debug)]
pub struct Global {
    pub mangle: String,
    pub init: i32,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub mangle: String,
    pub ret: Ty,
    pub params: usize,
    pub body: Vec<TStmt>,
    pub local_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Int32,
    NullableInt,
    Bool,
    Unit,
    Void,
    Null,
}

#[derive(Clone, Debug)]
pub enum TStmt {
    Local(usize, Option<TExpr>),
    Assign(usize, TExpr),
    StoreGlobal(String, TExpr),
    Expr(TExpr),
    If(TExpr, Vec<TStmt>, Vec<TStmt>),
    While(TExpr, Vec<TStmt>),
    Return(Option<TExpr>),
}

#[derive(Clone, Debug)]
pub enum TExpr {
    Int(i32),
    Bool(bool),
    Local(usize),
    Global(String),
    UnaryNeg(Box<TExpr>),
    Binary(BinOp, Box<TExpr>, Box<TExpr>),
    Call(String, Vec<TExpr>, Ty),
    Unit,
}

struct ClassRec {
    file: PathBuf,
    source: String,
    package: String,
    name: String,
    audience: Audience,
    is_open: bool,
    is_sealed: bool,
    extends: Option<String>,
    permits: Vec<String>,
    fields: Vec<Field>,
    methods: Vec<Method>,
}

pub fn check_project(units: &[Unit]) -> Result<Checked, Vec<Diagnostic>> {
    let mut errors = Vec::new();
    let mut classes = Vec::new();
    for unit in units {
        if unit.package == "cleat" || unit.package.starts_with("cleat.") {
            errors.push(diag(unit, 0, "a user package named cleat is rejected"));
        }
        let mut publics = 0;
        for ty in &unit.types {
            if ty.audience == Audience::Public {
                publics += 1;
                let stem = unit.file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if stem != ty.name {
                    errors.push(diag(
                        unit,
                        ty.span,
                        &format!(
                            "the public type {} must be declared in a file named {}.cleat",
                            ty.name, ty.name
                        ),
                    ));
                }
            }
            if ty.is_open && ty.is_sealed {
                errors.push(diag(
                    unit,
                    ty.span,
                    "a class cannot be both open and sealed",
                ));
            }
            if ty.is_sealed && ty.permits.is_empty() {
                errors.push(diag(
                    unit,
                    ty.span,
                    "a sealed class must name its permitted subclasses",
                ));
            }
            let mains = ty.methods.iter().filter(|m| m.name == "main").count();
            if mains > 1 {
                errors.push(diag(unit, ty.span, "a type declares at most one main"));
            }
            classes.push(ClassRec {
                file: unit.file.clone(),
                source: unit.source.clone(),
                package: unit.package.clone(),
                name: ty.name.clone(),
                audience: ty.audience,
                is_open: ty.is_open,
                is_sealed: ty.is_sealed,
                extends: ty.extends.clone(),
                permits: ty.permits.clone(),
                fields: ty.fields.clone(),
                methods: ty.methods.clone(),
            });
        }
        if publics > 1 {
            errors.push(diag(unit, 0, "a file contains at most one public type"));
        }
    }
    let mut by_qual: HashMap<String, usize> = HashMap::new();
    for (idx, class) in classes.iter().enumerate() {
        let qual = format!("{}.{}", class.package, class.name);
        if by_qual.insert(qual.clone(), idx).is_some() {
            errors.push(diag_class(class, 0, &format!("duplicate type {qual}")));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    for unit in units {
        for import in &unit.imports {
            if import.star {
                let package_has_type = classes.iter().any(|class| {
                    class.package == import.name && class.audience == Audience::Public
                });
                if !package_has_type {
                    errors.push(diag(
                        unit,
                        import.span,
                        &format!("import {}.* names no public type", import.name),
                    ));
                }
                continue;
            }
            match by_qual.get(&import.name) {
                Some(idx) if classes[*idx].audience == Audience::Public => {}
                Some(_) => errors.push(diag(
                    unit,
                    import.span,
                    &format!("import {} names a type that is not public", import.name),
                )),
                None => errors.push(diag(
                    unit,
                    import.span,
                    &format!(
                        "import {} does not name a type in this project",
                        import.name
                    ),
                )),
            }
        }
    }

    for class in &classes {
        if let Some(super_name) = &class.extends {
            let Some(sup) = resolve_simple(super_name, class, &classes, &by_qual, units) else {
                errors.push(diag_class(
                    class,
                    0,
                    &format!("cannot find superclass {super_name}"),
                ));
                continue;
            };
            if !visible(class, sup) {
                errors.push(diag_class(
                    class,
                    0,
                    &format!("superclass {} is not visible here", sup.name),
                ));
                continue;
            }
            if !sup.is_open && !sup.is_sealed {
                errors.push(diag_class(
                    class,
                    0,
                    &format!(
                        "{} is final, so {} cannot extend it (section 4.1)",
                        sup.name, class.name
                    ),
                ));
            }
            if sup.is_sealed {
                let child = format!("{}.{}", class.package, class.name);
                let permitted = sup.permits.iter().any(|name| {
                    resolve_simple(name, sup, &classes, &by_qual, units)
                        .map(|c| format!("{}.{}", c.package, c.name) == child)
                        .unwrap_or(false)
                });
                if !permitted {
                    errors.push(diag_class(
                        class,
                        0,
                        &format!("{} is sealed and does not permit {}", sup.name, class.name),
                    ));
                }
            }
        }
    }

    for class in &classes {
        for name in &class.permits {
            if resolve_simple(name, class, &classes, &by_qual, units).is_none() {
                errors.push(diag_class(
                    class,
                    0,
                    &format!(
                        "permits names {name}, which is not a type in this project (section 3.4)"
                    ),
                ));
            }
        }
        for method in &class.methods {
            note_only(
                class,
                &method.only,
                &classes,
                &by_qual,
                units,
                method.span,
                &mut errors,
            );
        }
        for field in &class.fields {
            note_only(
                class,
                &field.only,
                &classes,
                &by_qual,
                units,
                field.span,
                &mut errors,
            );
        }
    }

    let mut globals = Vec::new();
    for class in &classes {
        for field in &class.fields {
            match lower_field(class, field) {
                Ok(global) => globals.push(global),
                Err(err) => errors.push(err),
            }
        }
    }

    let mut funcs = Vec::new();
    for class in &classes {
        for method in &class.methods {
            if !method.is_static {
                errors.push(diag_class(
                    class,
                    method.span,
                    "instance methods need a receiver object; this milestone lowers static methods (section 4.2)",
                ));
                continue;
            }
            match check_method(class, method, &classes, &by_qual, units) {
                Ok(func) => funcs.push(func),
                Err(err) => errors.extend(err),
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Checked { funcs, globals })
}

fn note_only(
    class: &ClassRec,
    only: &[String],
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
    span: usize,
    errors: &mut Vec<Diagnostic>,
) {
    let mut seen = Vec::new();
    for name in only {
        if seen.contains(name) {
            errors.push(diag_class(
                class,
                span,
                &format!("only names {name} twice (section 3.2)"),
            ));
            continue;
        }
        seen.push(name.clone());
        if resolve_simple(name, class, classes, by_qual, units).is_none() {
            errors.push(diag_class(
                class,
                span,
                &format!(
                    "only names {name}, which is not a visible type in this project (section 3.2)"
                ),
            ));
        }
    }
}

fn lower_field(class: &ClassRec, field: &Field) -> Result<Global, Diagnostic> {
    if !field.is_static {
        return Err(diag_class(
            class,
            field.span,
            "an instance field is a slot of a reference object (section 9.6)",
        ));
    }
    if field.ty != "Int32" {
        return Err(diag_class(
            class,
            field.span,
            "a static field in this milestone is Int32 (section 6.3)",
        ));
    }
    let init = match &field.init {
        None if field.nullable => 0,
        None => {
            return Err(diag_class(
                class,
                field.span,
                "a non-null field has no default and must be initialized (section 5.2)",
            ));
        }
        Some(expr) => match const_int(expr) {
            Ok(value) => value,
            Err("null") if field.nullable => 0,
            Err("null") => {
                return Err(diag_class(
                    class,
                    field.span,
                    "a type written without @Nullable does not contain null (section 5.2)",
                ));
            }
            Err(message) => return Err(diag_class(class, field.span, message)),
        },
    };
    Ok(Global {
        mangle: field_mangle(&class.package, &class.name, &field.name),
        init,
    })
}

fn const_int(expr: &Expr) -> Result<i32, &'static str> {
    match &expr.kind {
        ExprKind::Int(n) => {
            i32::try_from(*n).map_err(|_| "this integer literal does not fit in Int32")
        }
        ExprKind::Null => Err("null"),
        ExprKind::UnaryNeg(inner) => {
            let ExprKind::Int(n) = &inner.kind else {
                return Err("a static Int32 initializer must be a constant (section 6.3)");
            };
            i32::try_from(-n)
                .map_err(|_| "constant Int32 negation raises ArithmeticException (section 6.3)")
        }
        ExprKind::Binary(op, left, right) => {
            let ExprKind::Int(a) = &left.kind else {
                return Err("a static Int32 initializer must be a constant (section 6.3)");
            };
            let ExprKind::Int(b) = &right.kind else {
                return Err("a static Int32 initializer must be a constant (section 6.3)");
            };
            fold_int(*a, *b, *op).map_err(|message| message)
        }
        _ => Err("a static Int32 initializer must be a constant (section 6.3)"),
    }
}

fn check_method(
    class: &ClassRec,
    method: &Method,
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
) -> Result<Func, Vec<Diagnostic>> {
    if method.is_open && method.is_static {
        return Err(vec![diag_class(
            class,
            method.span,
            "a static method is resolved on the compile-time class, so open is rejected (section 4.2)",
        )]);
    }
    let ret = match &method.result {
        ResultType::Void => Ty::Void,
        ResultType::Named { name, nullable } => {
            resolve_ty(name, *nullable, class, classes, by_qual, units, method.span)?
        }
    };
    let mut params = Vec::new();
    for param in &method.params {
        params.push(resolve_ty(
            &param.ty,
            param.nullable,
            class,
            classes,
            by_qual,
            units,
            method.span,
        )?);
    }
    let mut locals: Vec<(String, Ty)> = method
        .params
        .iter()
        .zip(params.iter())
        .map(|(p, ty)| (p.name.clone(), *ty))
        .collect();
    let mut errors = Vec::new();
    let body = check_block(
        &method.body,
        class,
        classes,
        by_qual,
        units,
        &mut locals,
        ret,
        &mut errors,
    );
    if ret != Ty::Void && !always_returns(&method.body) {
        errors.push(diag_class(
            class,
            method.span,
            "a non-void method must return on every path",
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Func {
        mangle: mangle(&class.package, &class.name, &method.name),
        ret,
        params: params.len(),
        body,
        local_count: locals.len(),
    })
}

fn check_block(
    block: &Block,
    class: &ClassRec,
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
    locals: &mut Vec<(String, Ty)>,
    ret: Ty,
    errors: &mut Vec<Diagnostic>,
) -> Vec<TStmt> {
    let mut out = Vec::new();
    let mut reachable = true;
    for stmt in &block.stmts {
        if !reachable {
            errors.push(diag_class(class, stmt_span(stmt), "unreachable statement"));
            continue;
        }
        match stmt {
            Stmt::Local {
                span,
                ty,
                name,
                nullable,
                init,
            } => {
                let ty = match resolve_ty(ty, *nullable, class, classes, by_qual, units, *span) {
                    Ok(ty) => ty,
                    Err(err) => {
                        errors.extend(err);
                        continue;
                    }
                };
                let init = match init {
                    Some(expr) => {
                        match check_expr(expr, Some(ty), class, classes, by_qual, units, locals) {
                            Ok((got, texpr)) => match stored(ty, got, texpr) {
                                Ok(texpr) => Some(texpr),
                                Err(message) => {
                                    errors.push(diag_class(class, *span, message));
                                    None
                                }
                            },
                            Err(err) => {
                                errors.extend(err);
                                None
                            }
                        }
                    }
                    None if ty == Ty::NullableInt => Some(TExpr::Int(0)),
                    None => None,
                };
                if locals.iter().any(|(existing, _)| existing == name) {
                    errors.push(diag_class(
                        class,
                        *span,
                        &format!("{name} hides another local or parameter"),
                    ));
                }
                let id = locals.len();
                locals.push((name.clone(), ty));
                out.push(TStmt::Local(id, init));
            }
            Stmt::Assign {
                span,
                name,
                owner,
                expr,
            } => {
                let local_id = if owner.is_none() {
                    locals.iter().position(|(n, _)| n == name)
                } else {
                    None
                };
                if local_id.is_none() {
                    match assign_global(
                        class,
                        name,
                        owner.as_deref(),
                        classes,
                        by_qual,
                        units,
                        *span,
                    ) {
                        Ok((mangle, ty)) => {
                            match check_expr(expr, Some(ty), class, classes, by_qual, units, locals)
                            {
                                Ok((got, texpr)) => match stored(ty, got, texpr) {
                                    Ok(texpr) => out.push(TStmt::StoreGlobal(mangle, texpr)),
                                    Err(message) => errors.push(diag_class(class, *span, message)),
                                },
                                Err(err) => errors.extend(err),
                            }
                        }
                        Err(err) => errors.push(err),
                    }
                    continue;
                }
                let id = local_id.unwrap();
                let ty = locals[id].1;
                match check_expr(expr, Some(ty), class, classes, by_qual, units, locals) {
                    Ok((got, texpr)) => match stored(ty, got, texpr) {
                        Ok(texpr) => out.push(TStmt::Assign(id, texpr)),
                        Err(message) => errors.push(diag_class(class, *span, message)),
                    },
                    Err(err) => errors.extend(err),
                }
            }
            Stmt::Expr { span, expr } => match check_expr(
                expr, None, class, classes, by_qual, units, locals,
            ) {
                Ok((Ty::Void, texpr)) => out.push(TStmt::Expr(texpr)),
                Ok((Ty::Unit, _)) => errors.push(diag_class(
                    class,
                    *span,
                    "a Unit result must be used; a void method is the form whose result is ignored",
                )),
                Ok(_) => errors.push(diag_class(
                    class,
                    *span,
                    "the result of this expression is unused",
                )),
                Err(err) => errors.extend(err),
            },
            Stmt::If {
                span,
                cond,
                then_body,
                else_body,
            } => {
                let cond = match check_expr(
                    cond,
                    Some(Ty::Bool),
                    class,
                    classes,
                    by_qual,
                    units,
                    locals,
                ) {
                    Ok((Ty::Bool, texpr)) => texpr,
                    Ok(_) => {
                        errors.push(diag_class(class, *span, "if requires a Boolean condition"));
                        TExpr::Bool(false)
                    }
                    Err(err) => {
                        errors.extend(err);
                        TExpr::Bool(false)
                    }
                };
                let then_stmts = check_block(
                    then_body, class, classes, by_qual, units, locals, ret, errors,
                );
                let else_stmts = else_body
                    .as_ref()
                    .map(|b| check_block(b, class, classes, by_qual, units, locals, ret, errors))
                    .unwrap_or_default();
                out.push(TStmt::If(cond, then_stmts, else_stmts));
            }
            Stmt::While { span, cond, body } => {
                let cond = match check_expr(
                    cond,
                    Some(Ty::Bool),
                    class,
                    classes,
                    by_qual,
                    units,
                    locals,
                ) {
                    Ok((Ty::Bool, texpr)) => texpr,
                    Ok(_) => {
                        errors.push(diag_class(
                            class,
                            *span,
                            "while requires a Boolean condition",
                        ));
                        TExpr::Bool(false)
                    }
                    Err(err) => {
                        errors.extend(err);
                        TExpr::Bool(false)
                    }
                };
                let body = check_block(body, class, classes, by_qual, units, locals, ret, errors);
                out.push(TStmt::While(cond, body));
            }
            Stmt::Return { span, expr } => {
                reachable = false;
                match (ret, expr) {
                    (Ty::Void, None) => out.push(TStmt::Return(None)),
                    (Ty::Void, Some(_)) => {
                        errors.push(diag_class(class, *span, "a void method returns no value"))
                    }
                    (Ty::Unit, None) => out.push(TStmt::Return(Some(TExpr::Unit))),
                    (_, Some(expr)) => match check_expr(
                        expr,
                        Some(match ret {
                            Ty::Void => Ty::Unit,
                            other => other,
                        }),
                        class,
                        classes,
                        by_qual,
                        units,
                        locals,
                    ) {
                        Ok((got, texpr)) if got == ret || (ret == Ty::Unit && got == Ty::Unit) => {
                            out.push(TStmt::Return(Some(texpr)))
                        }
                        Ok((got, _)) => errors.push(diag_class(
                            class,
                            *span,
                            &format!("return has type {got:?}, expected {ret:?}"),
                        )),
                        Err(err) => errors.extend(err),
                    },
                    (Ty::Int32 | Ty::NullableInt | Ty::Bool | Ty::Null, None) => {
                        errors.push(diag_class(class, *span, "this method must return a value"))
                    }
                }
            }
            Stmt::Block(inner) => {
                out.extend(check_block(
                    inner, class, classes, by_qual, units, locals, ret, errors,
                ));
            }
        }
    }
    out
}

fn check_expr(
    expr: &Expr,
    expected: Option<Ty>,
    class: &ClassRec,
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
    locals: &[(String, Ty)],
) -> Result<(Ty, TExpr), Vec<Diagnostic>> {
    match &expr.kind {
        ExprKind::Int(n) => {
            if expected == Some(Ty::Int32) || expected.is_none() {
                let Some(n) = i32::try_from(*n).ok() else {
                    return err(
                        class,
                        expr.span,
                        "this integer literal does not fit in Int32",
                    );
                };
                if expected.is_none() {
                    return err(
                        class,
                        expr.span,
                        "a bare numeric literal has type Number; write it where an Int32 is expected",
                    );
                }
                Ok((Ty::Int32, TExpr::Int(n)))
            } else {
                err(
                    class,
                    expr.span,
                    "this integer literal is not a Boolean or Unit",
                )
            }
        }
        ExprKind::Bool(v) => Ok((Ty::Bool, TExpr::Bool(*v))),
        ExprKind::Null => Ok((Ty::Null, TExpr::Int(0))),
        ExprKind::NewUnit => Ok((Ty::Unit, TExpr::Unit)),
        ExprKind::Name(name) => {
            if let Some(id) = locals.iter().rposition(|(n, _)| n == name) {
                if locals[id].1 == Ty::NullableInt {
                    return err(
                        class,
                        expr.span,
                        "a @Nullable Int32 must be narrowed before it is used as Int32 (section 5.3)",
                    );
                }
                return Ok((locals[id].1, TExpr::Local(id)));
            }
            if let Some(field) = class.fields.iter().find(|field| field.name == *name) {
                return load_field(class, class, field, expr.span, classes, by_qual, units);
            }
            if resolve_simple(name, class, classes, by_qual, units).is_some() {
                return err(class, expr.span, &format!("{name} is a type, not a value"));
            }
            err(class, expr.span, &format!("unknown name {name}"))
        }
        ExprKind::Select(recv, field_name) => {
            let ExprKind::Name(type_name) = &recv.kind else {
                return err(
                    class,
                    expr.span,
                    "a static field is named on its class (section 4.2)",
                );
            };
            let Some(owner) = resolve_simple(type_name, class, classes, by_qual, units) else {
                return err(class, expr.span, &format!("cannot find type {type_name}"));
            };
            let Some(field) = owner.fields.iter().find(|field| field.name == *field_name) else {
                return err(
                    class,
                    expr.span,
                    &format!("no static field {field_name} on {}", owner.name),
                );
            };
            load_field(class, owner, field, expr.span, classes, by_qual, units)
        }
        ExprKind::UnaryNeg(inner) => {
            if let ExprKind::Int(n) = &inner.kind {
                return match i32::try_from(-n) {
                    Ok(folded) => Ok((Ty::Int32, TExpr::Int(folded))),
                    Err(_) => err(
                        class,
                        expr.span,
                        "constant Int32 negation raises ArithmeticException (section 6.3)",
                    ),
                };
            }
            let (ty, texpr) = check_expr(
                inner,
                Some(Ty::Int32),
                class,
                classes,
                by_qual,
                units,
                locals,
            )?;
            if ty != Ty::Int32 {
                return err(class, expr.span, "negation requires Int32");
            }
            if let TExpr::Int(n) = texpr {
                let wide = -i64::from(n);
                let Some(folded) = i32::try_from(wide).ok() else {
                    return err(
                        class,
                        expr.span,
                        "constant Int32 negation is not representable",
                    );
                };
                return Ok((Ty::Int32, TExpr::Int(folded)));
            }
            Ok((Ty::Int32, TExpr::UnaryNeg(Box::new(texpr))))
        }
        ExprKind::Binary(op, left, right) => check_binary(
            *op, left, right, expected, expr.span, class, classes, by_qual, units, locals,
        ),
        ExprKind::Call(callee, args) => check_call(
            callee, args, class, classes, by_qual, units, locals, expr.span,
        ),
    }
}

fn check_binary(
    op: BinOp,
    left: &Expr,
    right: &Expr,
    expected: Option<Ty>,
    span: usize,
    class: &ClassRec,
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
    locals: &[(String, Ty)],
) -> Result<(Ty, TExpr), Vec<Diagnostic>> {
    let arith = matches!(
        op,
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
    );
    if arith {
        if let (ExprKind::Int(a), ExprKind::Int(b)) = (&left.kind, &right.kind) {
            if expected != Some(Ty::Int32) {
                return err(
                    class,
                    span,
                    "a fold of bare literals has type Number unless an Int32 is expected",
                );
            }
            let folded =
                fold_int(*a, *b, op).map_err(|message| vec![diag_class(class, span, message)])?;
            return Ok((Ty::Int32, TExpr::Int(folded)));
        }
    }
    let (lt, lexpr) = check_expr(
        left,
        if arith { Some(Ty::Int32) } else { None },
        class,
        classes,
        by_qual,
        units,
        locals,
    )?;
    let (rt, rexpr) = check_expr(
        right,
        if arith { Some(Ty::Int32) } else { Some(lt) },
        class,
        classes,
        by_qual,
        units,
        locals,
    )?;
    if arith {
        if lt != Ty::Int32 || rt != Ty::Int32 {
            return err(
                class,
                span,
                "Int32 arithmetic does not accept Number or another fixed width (section 6.3)",
            );
        }
        if let (TExpr::Int(a), TExpr::Int(b)) = (&lexpr, &rexpr) {
            let folded = fold_int(i128::from(*a), i128::from(*b), op)
                .map_err(|message| vec![diag_class(class, span, message)])?;
            return Ok((Ty::Int32, TExpr::Int(folded)));
        }
        return Ok((
            Ty::Int32,
            TExpr::Binary(op, Box::new(lexpr), Box::new(rexpr)),
        ));
    }
    if lt != rt || (lt != Ty::Int32 && lt != Ty::Bool) {
        return err(
            class,
            span,
            "the two sides of a comparison have different types",
        );
    }
    Ok((
        Ty::Bool,
        TExpr::Binary(op, Box::new(lexpr), Box::new(rexpr)),
    ))
}

fn fold_int(a: i128, b: i128, op: BinOp) -> Result<i32, &'static str> {
    let a = i32::try_from(a).map_err(|_| "this integer literal does not fit in Int32")?;
    let b = i32::try_from(b).map_err(|_| "this integer literal does not fit in Int32")?;
    let wide = match op {
        BinOp::Add => i64::from(a) + i64::from(b),
        BinOp::Sub => i64::from(a) - i64::from(b),
        BinOp::Mul => i64::from(a) * i64::from(b),
        BinOp::Div => {
            if b == 0 {
                return Err(
                    "constant Int32 division by zero raises ArithmeticException (section 6.3)",
                );
            }
            if a == i32::MIN && b == -1 {
                return Err(
                    "constant Int32 division of the minimum by -1 raises ArithmeticException (section 6.3)",
                );
            }
            if a % b != 0 {
                return Err(
                    "constant Int32 div raises ArithmeticException unless the division is exact (section 6.3)",
                );
            }
            i64::from(a / b)
        }
        BinOp::Rem => {
            if b == 0 {
                return Err(
                    "constant Int32 remainder by zero raises ArithmeticException (section 6.3)",
                );
            }
            i64::from(a % b)
        }
        _ => return Err("not arithmetic"),
    };
    i32::try_from(wide).map_err(|_| "constant Int32 arithmetic is not representable and raises ArithmeticException (section 6.3)")
}

fn check_call(
    callee: &Expr,
    args: &[Expr],
    class: &ClassRec,
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
    locals: &[(String, Ty)],
    span: usize,
) -> Result<(Ty, TExpr), Vec<Diagnostic>> {
    let (owner, method_name) = match &callee.kind {
        ExprKind::Name(name) => (class, name.clone()),
        ExprKind::Select(recv, name) => {
            let ExprKind::Name(type_name) = &recv.kind else {
                return err(class, span, "a call receiver must be a type name");
            };
            let Some(owner) = resolve_simple(type_name, class, classes, by_qual, units) else {
                if let Some(hidden) = hidden_in_package(type_name, class, classes) {
                    return err(
                        class,
                        span,
                        &format!(
                            "type {} is private to its file and is not visible here",
                            hidden.name
                        ),
                    );
                }
                return err(class, span, &format!("cannot find type {type_name}"));
            };
            if !visible(class, owner) {
                return err(
                    class,
                    span,
                    &format!("type {} is not visible from this file", owner.name),
                );
            }
            (owner, name.clone())
        }
        _ => return err(class, span, "unsupported call"),
    };
    let Some(method) = owner
        .methods
        .iter()
        .find(|m| m.name == method_name && m.is_static)
    else {
        return err(
            class,
            span,
            &format!("no static method {method_name} on {}", owner.name),
        );
    };
    if !method_visible(class, owner, method) {
        let section = if method.only.is_empty() { "3.1" } else { "3.2" };
        return err(
            class,
            span,
            &format!("method {} is not visible here (section {section})", method.name),
        );
    }
    if method.params.len() != args.len() {
        return err(class, span, "wrong number of arguments");
    }
    let mut targs = Vec::new();
    for (param, arg) in method.params.iter().zip(args) {
        let pty = resolve_ty(
            &param.ty,
            param.nullable,
            owner,
            classes,
            by_qual,
            units,
            param_span(method),
        )?;
        let (got, texpr) = check_expr(arg, Some(pty), class, classes, by_qual, units, locals)?;
        if !fits(pty, got) {
            return err(
                class,
                arg.span,
                &format!("argument has type {got:?}, expected {pty:?}"),
            );
        }
        targs.push(texpr);
    }
    let ret = match &method.result {
        ResultType::Void => Ty::Void,
        ResultType::Named { name, nullable } => {
            resolve_ty(name, *nullable, owner, classes, by_qual, units, method.span)?
        }
    };
    let mangle = mangle(&owner.package, &owner.name, &method.name);
    Ok((ret, TExpr::Call(mangle, targs, ret)))
}

fn resolve_ty(
    name: &str,
    nullable: bool,
    from: &ClassRec,
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
    span: usize,
) -> Result<Ty, Vec<Diagnostic>> {
    match (name, nullable) {
        ("Int32", true) => Ok(Ty::NullableInt),
        ("Int32", false) => Ok(Ty::Int32),
        ("Boolean", false) => Ok(Ty::Bool),
        ("Unit", false) => Ok(Ty::Unit),
        (_, true) => err(
            from,
            span,
            "@Nullable in this milestone is lowered for Int32 (section 5.2)",
        ),
        (other, false) => {
            if resolve_simple(other, from, classes, by_qual, units).is_some() {
                err(
                    from,
                    span,
                    &format!("{other} is not an Int32, Boolean, or Unit in this milestone"),
                )
            } else {
                err(from, span, &format!("unknown type {other}"))
            }
        }
    }
}

fn fits(dest: Ty, got: Ty) -> bool {
    dest == got || (dest == Ty::NullableInt && matches!(got, Ty::Int32 | Ty::Null))
}

fn stored(dest: Ty, got: Ty, texpr: TExpr) -> Result<TExpr, &'static str> {
    if got == Ty::Null && dest != Ty::NullableInt {
        return Err("a type written without @Nullable does not contain null (section 5.2)");
    }
    if fits(dest, got) {
        Ok(texpr)
    } else {
        Err("the value is not assignable to this type (section 4.4)")
    }
}

fn only_allows(from: &ClassRec, owner: &ClassRec, only: &[String]) -> bool {
    if only.is_empty() {
        return true;
    }
    (from.name == owner.name && from.package == owner.package)
        || only.iter().any(|name| name == &from.name)
}

fn load_field(
    from: &ClassRec,
    owner: &ClassRec,
    field: &Field,
    span: usize,
    _classes: &[ClassRec],
    _by_qual: &HashMap<String, usize>,
    _units: &[Unit],
) -> Result<(Ty, TExpr), Vec<Diagnostic>> {
    if !field.is_static {
        return err(
            from,
            span,
            "an instance field is a slot of a reference object (section 9.6)",
        );
    }
    if !visible(from, owner) || !field_visible(from, owner, field) {
        return err(
            from,
            span,
            &format!("field {} is not visible here (section 3.2)", field.name),
        );
    }
    if field.nullable {
        return err(
            from,
            span,
            "a @Nullable Int32 must be narrowed before it is used as Int32 (section 5.3)",
        );
    }
    Ok((
        Ty::Int32,
        TExpr::Global(field_mangle(&owner.package, &owner.name, &field.name)),
    ))
}

fn assign_global(
    from: &ClassRec,
    name: &str,
    owner_name: Option<&str>,
    classes: &[ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
    span: usize,
) -> Result<(String, Ty), Diagnostic> {
    let owner = if let Some(owner_name) = owner_name {
        resolve_simple(owner_name, from, classes, by_qual, units)
            .ok_or_else(|| diag_class(from, span, &format!("cannot find type {owner_name}")))?
    } else {
        from
    };
    let Some(field) = owner.fields.iter().find(|field| field.name == name) else {
        return Err(diag_class(from, span, &format!("unknown local {name}")));
    };
    if !field.is_static {
        return Err(diag_class(
            from,
            span,
            "an instance field is a slot of a reference object (section 9.6)",
        ));
    }
    if !visible(from, owner) || !field_visible(from, owner, field) {
        return Err(diag_class(
            from,
            span,
            &format!("field {} is not visible here (section 3.2)", field.name),
        ));
    }
    let ty = if field.nullable {
        Ty::NullableInt
    } else {
        Ty::Int32
    };
    Ok((field_mangle(&owner.package, &owner.name, &field.name), ty))
}

fn field_visible(from: &ClassRec, owner: &ClassRec, field: &Field) -> bool {
    if !only_allows(from, owner, &field.only) {
        return false;
    }
    match field.audience {
        Audience::Private => from.file == owner.file && from.name == owner.name,
        Audience::Package => from.package == owner.package,
        Audience::Public => true,
    }
}

pub fn field_mangle(package: &str, ty: &str, field: &str) -> String {
    format!("g_{}_{}_{}", package.replace('.', "_"), ty, field)
}

fn resolve_simple<'a>(
    name: &str,
    from: &ClassRec,
    classes: &'a [ClassRec],
    by_qual: &HashMap<String, usize>,
    units: &[Unit],
) -> Option<&'a ClassRec> {
    if let Some(found) = classes
        .iter()
        .find(|c| c.file == from.file && c.name == name)
    {
        return Some(found);
    }
    let unit = units.iter().find(|u| u.file == from.file)?;
    for import in &unit.imports {
        if import.star {
            continue;
        }
        if import.name.rsplit('.').next() == Some(name) {
            if let Some(idx) = by_qual.get(&import.name) {
                let class = &classes[*idx];
                if visible(from, class) {
                    return Some(class);
                }
            }
        }
    }
    let mut same_package = None;
    for class in classes {
        if class.package == from.package && class.name == name && visible(from, class) {
            if same_package.is_some() {
                return None;
            }
            same_package = Some(class);
        }
    }
    if same_package.is_some() {
        return same_package;
    }
    for import in &unit.imports {
        if !import.star {
            continue;
        }
        let qual = format!("{}.{}", import.name, name);
        if let Some(idx) = by_qual.get(&qual) {
            let class = &classes[*idx];
            if visible(from, class) {
                return Some(class);
            }
        }
    }
    None
}

fn hidden_in_package<'a>(
    name: &str,
    from: &ClassRec,
    classes: &'a [ClassRec],
) -> Option<&'a ClassRec> {
    classes.iter().find(|class| {
        class.package == from.package
            && class.name == name
            && class.audience == Audience::Private
            && class.file != from.file
    })
}

fn visible(from: &ClassRec, target: &ClassRec) -> bool {
    match target.audience {
        Audience::Private => from.file == target.file,
        Audience::Package => from.package == target.package,
        Audience::Public => true,
    }
}

fn method_visible(from: &ClassRec, owner: &ClassRec, method: &Method) -> bool {
    if !visible(from, owner) {
        return false;
    }
    if !only_allows(from, owner, &method.only) {
        return false;
    }
    match method.audience {
        Audience::Private => from.file == owner.file && from.name == owner.name,
        Audience::Package => from.package == owner.package,
        Audience::Public => true,
    }
}

fn param_span(method: &Method) -> usize {
    method.span
}

fn always_returns(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        Stmt::Return { .. } => true,
        Stmt::If {
            then_body,
            else_body,
            ..
        } => else_body
            .as_ref()
            .is_some_and(|e| always_returns(then_body) && always_returns(e)),
        Stmt::Block(inner) => always_returns(inner),
        _ => false,
    })
}

fn stmt_span(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::Local { span, .. }
        | Stmt::Expr { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Return { span, .. } => *span,
        Stmt::Block(_) => 0,
    }
}

pub fn mangle(package: &str, ty: &str, method: &str) -> String {
    format!("m_{}_{}_{}", package.replace('.', "_"), ty, method)
}

fn diag(unit: &Unit, span: usize, message: &str) -> Diagnostic {
    let (line, column) = line_col(&unit.source, span);
    Diagnostic {
        file: unit.file.clone(),
        line,
        column,
        message: message.to_string(),
    }
}

fn diag_class(class: &ClassRec, span: usize, message: &str) -> Diagnostic {
    let (line, column) = line_col(&class.source, span);
    Diagnostic {
        file: class.file.clone(),
        line,
        column,
        message: message.to_string(),
    }
}

fn err<T>(class: &ClassRec, span: usize, message: &str) -> Result<T, Vec<Diagnostic>> {
    Err(vec![diag_class(class, span, message)])
}
