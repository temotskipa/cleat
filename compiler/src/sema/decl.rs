//! Builds the class table from the syntax trees: declarations, headers, members.

use super::program::*;
use super::types::*;
use crate::ast::{self, Audience, Member, TypeKind, Variance};
use crate::lex::Pos;
use crate::parse;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// The names in scope where a type is written.
#[derive(Clone, Debug)]
pub struct TypeCx {
    pub unit: usize,
    /// The class whose declaration contains the type, when there is one.
    pub class: Option<ClassId>,
    pub tvars: Vec<(String, Tv)>,
}

impl TypeCx {
    pub fn for_class(p: &Program, id: ClassId, with_tparams: bool) -> TypeCx {
        let c = p.class(id);
        let mut tvars = Vec::new();
        if with_tparams {
            for (i, tp) in c.tparams.iter().enumerate() {
                tvars.push((tp.name.clone(), Tv { owner: TvOwner::Class(id), index: i as u32 }));
            }
        }
        TypeCx { unit: c.unit, class: Some(id), tvars }
    }
}

const WELL_KNOWN: &[&str] = &[
    "Object", "Null", "Unit", "Boolean", "Char", "String", "Int8", "Int16", "Int32", "Int",
    "UInt8", "UInt16", "UInt32", "UInt64", "Float32", "Float64", "Rational", "Array", "Iterable",
    "Iterator", "Throwable", "Enum", "Class", "Annotation", "Site", "Nullable", "Refines",
    "Widens", "Target", "Inherited", "Narrows", "Override", "Discardable", "Implicit",
    "Deprecated", "Intrinsic", "Symbol", "Pointer", "AssertionException",
    "IllegalArgumentException",
];

pub fn build(mut units: Vec<ast::Unit>) -> Program {
    synthesize_enum_members(&mut units);
    let mut p = Program {
        units,
        classes: Vec::new(),
        by_name: HashMap::new(),
        packages: HashSet::new(),
        wk: WellKnown::default(),
        diags: Vec::new(),
        warnings: Vec::new(),
        captures: Vec::new(),
        supertype_cache: HashMap::new(),
        headers_done: false,
        deferred_type_checks: Vec::new(),
    };
    collect(&mut p);
    if !well_known(&mut p) {
        return p;
    }
    qualifier_kinds(&mut p);
    headers(&mut p);
    p.headers_done = true;
    let deferred = std::mem::take(&mut p.deferred_type_checks);
    for (t, unit, pos) in deferred {
        check_type_args(&mut p, &t, unit, pos);
    }
    members(&mut p);
    hierarchy(&mut p);
    p
}

/// Gives every enum its `values` and `valueOf`, written as source and parsed.
fn synthesize_enum_members(units: &mut [ast::Unit]) {
    for unit in units.iter_mut() {
        for d in unit.types.iter_mut() {
            if d.kind != TypeKind::Enum {
                continue;
            }
            let n = &d.name;
            let names: Vec<&str> = d.constants.iter().map(|c| c.name.as_str()).collect();
            let text = format!(
                "class {n} {{\n\
                 public static {n}[] values() {{ return new {n}[] {{ {list} }}; }}\n\
                 public static {n} valueOf(String name) {{\n\
                 for ({n} constant : values()) {{ if (constant.name() == name) {{ return constant; }} }}\n\
                 throw new IllegalArgumentException(\"{n} has no constant \" + name);\n\
                 }}\n}}\n",
                list = names.join(", ")
            );
            if let Ok(parsed) = parse::parse_unit(Path::new("<enum>"), &text) {
                let pos = d.pos;
                for mut m in parsed.types.into_iter().next().unwrap().members {
                    if let Member::Method(md) = &mut m {
                        md.pos = pos;
                    }
                    d.members.push(m);
                }
            }
        }
    }
}

fn collect(p: &mut Program) {
    for ui in 0..p.units.len() {
        let package = p.units[ui].package.join(".");
        let mut prefix = String::new();
        for seg in p.units[ui].package.clone() {
            if !prefix.is_empty() {
                prefix.push('.');
            }
            prefix.push_str(&seg);
            p.packages.insert(prefix.clone());
        }
        let stem = p.units[ui]
            .file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut public_seen = false;
        for di in 0..p.units[ui].types.len() {
            let d = p.units[ui].types[di].clone();
            let aud = match d.mods.audience {
                None => Aud::File,
                Some(Audience::Package) => Aud::Package,
                Some(Audience::Public) => Aud::Public,
                Some(_) => {
                    p.error(ui, d.pos, "a type is `public`, `package`, or visible to its file only");
                    Aud::File
                }
            };
            if d.mods.only.is_some() {
                p.error(ui, d.pos, "`only` is not written on a type");
            }
            if aud == Aud::Public {
                if public_seen {
                    p.error(ui, d.pos, "a file declares at most one public type");
                }
                public_seen = true;
                if stem != d.name {
                    p.error(
                        ui,
                        d.pos,
                        format!("the public type `{}` must be in a file named `{}.cleat`", d.name, d.name),
                    );
                }
            }
            let id = p.classes.len() as ClassId;
            let qname = if package.is_empty() { d.name.clone() } else { format!("{package}.{}", d.name) };
            let key = (package.clone(), d.name.clone());
            let clash = p.by_name.get(&key).cloned().unwrap_or_default().into_iter().any(|other| {
                let o = p.class(other);
                o.unit == ui || (o.aud != Aud::File && aud != Aud::File)
            });
            if clash {
                p.error(ui, d.pos, format!("the type `{}` is declared twice", qname));
            }
            p.by_name.entry(key).or_default().push(id);
            let m = &d.mods;
            if m.is_static {
                p.error(ui, d.pos, "`static` is not written on a type");
            }
            if m.is_foreign {
                p.error(ui, d.pos, "`foreign` is written on a method");
            }
            let is_class = d.kind == TypeKind::Class;
            if (m.is_open || m.is_sealed || m.is_abstract) && d.kind == TypeKind::ValueClass {
                p.error(ui, d.pos, "a value class is final: `open`, `sealed` and `abstract` are rejected on it");
            }
            if m.is_abstract && !is_class {
                if d.kind != TypeKind::ValueClass {
                    p.error(ui, d.pos, "`abstract` is written on a class");
                }
            }
            if m.is_final && (m.is_open || m.is_abstract) {
                p.error(ui, d.pos, "`final` contradicts `open` and `abstract`");
            }
            if m.is_final && d.kind == TypeKind::Interface {
                p.error(ui, d.pos, "`final interface` is rejected");
            }
            if m.is_sealed && m.is_open {
                p.error(ui, d.pos, "`sealed open` is rejected");
            }
            if (m.is_open || m.is_sealed) && matches!(d.kind, TypeKind::Enum | TypeKind::Annotation) {
                p.error(ui, d.pos, "an enum and an annotation are final");
            }
            if m.is_sealed && d.permits.as_ref().map(|l| l.is_empty()).unwrap_or(true) {
                p.error(ui, d.pos, "a sealed type names the types it permits");
            }
            if !m.is_sealed && d.permits.is_some() {
                p.error(ui, d.pos, "`permits` is written on a sealed type");
            }
            p.classes.push(Class {
                id,
                unit: ui,
                name: d.name.clone(),
                package: package.clone(),
                qname,
                kind: d.kind,
                aud,
                is_open: m.is_open || (d.kind == TypeKind::Interface && !m.is_sealed),
                is_sealed: m.is_sealed,
                is_abstract: m.is_abstract,
                tparams: Vec::new(),
                superclass: None,
                interfaces: Vec::new(),
                permits: None,
                fields: Vec::new(),
                methods: Vec::new(),
                ctors: Vec::new(),
                static_inits: Vec::new(),
                static_init_checked: None,
                elements: Vec::new(),
                qualifier: Qualifier::No,
                refines_above: Vec::new(),
                targets: None,
                inherited: false,
                ann_uses: m.annotations.clone(),
                anns: Vec::new(),
                deprecated: None,
                lambda: None,
                pos: d.pos,
                decl_index: di,
            });
        }
    }
}

fn well_known(p: &mut Program) -> bool {
    let mut ids = HashMap::new();
    for name in WELL_KNOWN {
        let found = p
            .by_name
            .get(&("cleat".to_string(), name.to_string()))
            .and_then(|v| v.iter().copied().find(|id| p.class(*id).aud == Aud::Public));
        match found {
            Some(id) => {
                ids.insert(*name, id);
            }
            None => {
                p.diags.push(crate::Diagnostic::general(
                    Path::new("prelude"),
                    format!("the prelude does not declare `cleat.{name}`"),
                ));
                return false;
            }
        }
    }
    let g = |n: &str| ids[n];
    p.wk = WellKnown {
        object: g("Object"),
        null: g("Null"),
        unit: g("Unit"),
        boolean: g("Boolean"),
        char_: g("Char"),
        string: g("String"),
        int8: g("Int8"),
        int16: g("Int16"),
        int32: g("Int32"),
        int: g("Int"),
        uint8: g("UInt8"),
        uint16: g("UInt16"),
        uint32: g("UInt32"),
        uint64: g("UInt64"),
        float32: g("Float32"),
        float64: g("Float64"),
        rational: g("Rational"),
        array: g("Array"),
        iterable: g("Iterable"),
        iterator: g("Iterator"),
        throwable: g("Throwable"),
        enum_: g("Enum"),
        class: g("Class"),
        annotation: g("Annotation"),
        site: g("Site"),
        nullable: g("Nullable"),
        refines: g("Refines"),
        widens: g("Widens"),
        target: g("Target"),
        inherited: g("Inherited"),
        narrows: g("Narrows"),
        override_: g("Override"),
        discardable: g("Discardable"),
        implicit: g("Implicit"),
        deprecated: g("Deprecated"),
        intrinsic: g("Intrinsic"),
        symbol: g("Symbol"),
        pointer: g("Pointer"),
        assertion_exception: g("AssertionException"),
        illegal_argument_exception: g("IllegalArgumentException"),
    };
    true
}

// ---- looking up types by name ----

/// Whether code in `unit` may name the class.
pub fn type_visible(p: &Program, id: ClassId, unit: usize) -> bool {
    let c = p.class(id);
    match c.aud {
        Aud::Public => true,
        Aud::Package => p.units[unit].package.join(".") == c.package,
        _ => c.unit == unit,
    }
}

fn find_in_package(p: &Program, package: &str, name: &str, unit: usize) -> Option<ClassId> {
    let list = p.by_name.get(&(package.to_string(), name.to_string()))?;
    // A file-private type of this file comes before a wider type of the same name.
    list.iter()
        .copied()
        .find(|id| p.class(*id).unit == unit && p.class(*id).aud == Aud::File)
        .or_else(|| list.iter().copied().find(|id| p.class(*id).aud != Aud::File))
}

/// Looks up a simple type name in the order of section 1.6, without type parameters.
pub fn lookup_simple(p: &mut Program, unit: usize, name: &str, pos: Pos, report: bool) -> Option<ClassId> {
    let package = p.units[unit].package.join(".");
    // The same file.
    if let Some(list) = p.by_name.get(&(package.clone(), name.to_string())) {
        if let Some(id) = list.iter().copied().find(|id| p.class(*id).unit == unit) {
            return Some(id);
        }
    }
    // A type imported by name.
    let imports = p.units[unit].imports.clone();
    for imp in imports.iter().filter(|i| !i.star) {
        if imp.path.last().map(|s| s.as_str()) == Some(name) {
            let pkg = imp.path[..imp.path.len() - 1].join(".");
            if let Some(id) = find_in_package(p, &pkg, name, unit) {
                if !type_visible(p, id, unit) {
                    if report {
                        p.error(unit, imp.pos, format!("`{}` is not visible here", imp.path.join(".")));
                    }
                    return None;
                }
                return Some(id);
            }
        }
    }
    // The same package.
    if let Some(id) = find_in_package(p, &package, name, unit) {
        if type_visible(p, id, unit) {
            return Some(id);
        }
    }
    // A type imported by `*`.
    let mut starred = Vec::new();
    for imp in imports.iter().filter(|i| i.star) {
        let pkg = imp.path.join(".");
        if let Some(id) = find_in_package(p, &pkg, name, unit) {
            if p.class(id).aud == Aud::Public && !starred.contains(&id) {
                starred.push(id);
            }
        }
    }
    if starred.len() > 1 {
        if report {
            p.error(unit, pos, format!("`{name}` is imported from two packages; write its qualified name"));
        }
        return None;
    }
    if let Some(id) = starred.first() {
        return Some(*id);
    }
    // The prelude.
    if let Some(id) = find_in_package(p, "cleat", name, unit) {
        if type_visible(p, id, unit) {
            return Some(id);
        }
    }
    None
}

/// Looks up a possibly qualified type name.
pub fn lookup_class(p: &mut Program, unit: usize, name: &[String], pos: Pos, report: bool) -> Option<ClassId> {
    if name.len() == 1 {
        let found = lookup_simple(p, unit, &name[0], pos, report);
        if found.is_none() && report {
            p.error(unit, pos, format!("there is no type named `{}` here", name[0]));
        }
        return found;
    }
    let pkg = name[..name.len() - 1].join(".");
    let simple = &name[name.len() - 1];
    match find_in_package(p, &pkg, simple, unit) {
        Some(id) if type_visible(p, id, unit) => Some(id),
        Some(_) => {
            if report {
                p.error(unit, pos, format!("`{}` is not visible here", name.join(".")));
            }
            None
        }
        None => {
            if report {
                p.error(unit, pos, format!("there is no type named `{}`", name.join(".")));
            }
            None
        }
    }
}

/// Splits annotations into qualifiers, which are applied to `ty`, and declaration
/// annotations, which are returned. With `decl` false a declaration annotation is an error.
pub fn apply_annotations(
    p: &mut Program,
    cx: &TypeCx,
    ty: &mut Type,
    anns: &[ast::AnnotationUse],
    decl: bool,
) -> Vec<ast::AnnotationUse> {
    let mut rest = Vec::new();
    for a in anns {
        let Some(id) = lookup_class(p, cx.unit, &a.name, a.pos, true) else { continue };
        if !p.class(id).is_annotation() {
            p.error(cx.unit, a.pos, format!("`{}` is not an annotation", a.name.join(".")));
            continue;
        }
        if p.class(id).qualifier == Qualifier::No {
            if decl {
                rest.push(a.clone());
            } else {
                p.error(
                    cx.unit,
                    a.pos,
                    format!("`@{}` is not a qualifier, so it is not written on a type", p.class(id).name),
                );
            }
            continue;
        }
        if !matches!(a.args, ast::AnnArgs::None) {
            p.error(cx.unit, a.pos, "a qualifier has no elements");
        }
        if id == p.wk.nullable {
            if ty.nullable && !decl {
                // Written twice in one position.
                p.error(cx.unit, a.pos, "an annotation is written at most once in one position");
            }
            ty.nullable = true;
        } else {
            if ty.quals.contains(&id) && !decl {
                p.error(cx.unit, a.pos, "an annotation is written at most once in one position");
            }
            ty.add_qual(id);
        }
    }
    rest
}

pub fn resolve_type(p: &mut Program, cx: &TypeCx, t: &ast::TypeRef) -> Type {
    resolve_type_anns(p, cx, t, &[], false).0
}

/// Resolves a type written on a declaration. `mods_anns` are the annotations among the
/// declaration's modifiers: the qualifiers among them join the type, and the rest are
/// returned as the declaration's own.
pub fn resolve_type_anns(
    p: &mut Program,
    cx: &TypeCx,
    t: &ast::TypeRef,
    mods_anns: &[ast::AnnotationUse],
    decl: bool,
) -> (Type, Vec<ast::AnnotationUse>) {
    let mut ty = resolve_named(p, cx, t);
    let mut rest = apply_annotations(p, cx, &mut ty, &t.annotations, false);
    rest.extend(apply_annotations(p, cx, &mut ty, mods_anns, decl));
    // Brackets: the last pair is the innermost array.
    for dim in t.dims.iter().rev() {
        let mut arr = p.array_of(ty);
        apply_annotations(p, cx, &mut arr, dim, false);
        ty = arr;
    }
    if p.headers_done {
        check_type_args(p, &ty, cx.unit, t.pos);
    } else {
        p.deferred_type_checks.push((ty.clone(), cx.unit, t.pos));
    }
    (ty, rest)
}

fn resolve_named(p: &mut Program, cx: &TypeCx, t: &ast::TypeRef) -> Type {
    if t.name.len() == 1 {
        if let Some((_, tv)) = cx.tvars.iter().rev().find(|(n, _)| *n == t.name[0]) {
            if t.args.is_some() {
                p.error(cx.unit, t.pos, "a type parameter takes no type arguments");
            }
            return Type::var(*tv);
        }
    }
    let Some(id) = lookup_class(p, cx.unit, &t.name, t.pos, true) else { return Type::error() };
    let nparams = p.units[p.class(id).unit].types[p.class(id).decl_index].type_params.len();
    let written = t.args.as_ref().map(|a| a.len()).unwrap_or(0);
    if written != nparams {
        let name = p.class(id).name.clone();
        if written == 0 {
            p.error(
                cx.unit,
                t.pos,
                format!("`{name}` is generic and is written with its type arguments; there are no raw types"),
            );
        } else {
            p.error(cx.unit, t.pos, format!("`{name}` takes {nparams} type arguments, not {written}"));
        }
        return Type::error();
    }
    let mut args = Vec::new();
    for (i, a) in t.args.iter().flatten().enumerate() {
        let variance = p.units[p.class(id).unit].types[p.class(id).decl_index].type_params[i].variance;
        match a {
            ast::TypeArg::Type(tr) => args.push(Arg::Ty(resolve_type(p, cx, tr))),
            ast::TypeArg::Wildcard(None, _) => args.push(Arg::Wild(None, None)),
            ast::TypeArg::Wildcard(Some((is_super, tr)), wpos) => {
                let b = resolve_type(p, cx, tr);
                match (variance, is_super) {
                    (Variance::Out, true) => {
                        p.error(cx.unit, *wpos, "`? super` is rejected for a parameter declared `out`");
                        args.push(Arg::Ty(Type::error()));
                    }
                    (Variance::In, false) => {
                        p.error(cx.unit, *wpos, "`? extends` is rejected for a parameter declared `in`");
                        args.push(Arg::Ty(Type::error()));
                    }
                    // On a variant parameter the bounded wildcard is the same type as its bound.
                    (Variance::Out, false) | (Variance::In, true) => args.push(Arg::Ty(b)),
                    (Variance::Invariant, true) => args.push(Arg::Wild(None, Some(Box::new(b)))),
                    (Variance::Invariant, false) => args.push(Arg::Wild(Some(Box::new(b)), None)),
                }
            }
        }
    }
    Type::class(id, args)
}

/// Checks that each type argument is within the bound of its parameter and carries the
/// tags the parameter requires.
pub fn check_type_args(p: &mut Program, t: &Type, unit: usize, pos: Pos) {
    let Ty::Class(id, args) = &t.ty else { return };
    let id = *id;
    if args.len() != p.class(id).tparams.len() {
        return;
    }
    let subst = Subst::for_class(id, args);
    for (i, a) in args.iter().enumerate() {
        let tp = p.class(id).tparams[i].clone();
        let checked: Vec<Type> = match a {
            Arg::Ty(at) => {
                check_type_args(p, at, unit, pos);
                vec![at.clone()]
            }
            Arg::Wild(e, s) => {
                let mut v = Vec::new();
                if let Some(e) = e {
                    check_type_args(p, e, unit, pos);
                    v.push((**e).clone());
                }
                if let Some(s) = s {
                    check_type_args(p, s, unit, pos);
                    v.push((**s).clone());
                }
                v
            }
        };
        for at in &checked {
            if at.is_error() {
                continue;
            }
            for b in &tp.bounds {
                let b = subst.apply(b);
                if !p.is_subtype(at, &b) {
                    let (sa, sb, sn) = (p.show(at), p.show(&b), tp.name.clone());
                    p.error(
                        unit,
                        pos,
                        format!("`{sa}` is not within the bound `{sb}` of the type parameter `{sn}`"),
                    );
                }
            }
            for tag in &tp.tags {
                if !carries_tag(p, at, *tag) {
                    let (sa, st) = (p.show(at), p.class(*tag).name.clone());
                    p.error(unit, pos, format!("`{sa}` does not carry the tag `@{st}` that the type parameter requires"));
                }
            }
        }
    }
}

/// Whether the declaration of a type carries a tag, written or inherited.
pub fn carries_tag(p: &mut Program, t: &Type, tag: ClassId) -> bool {
    match &t.ty {
        Ty::Class(id, _) => class_carries(p, *id, tag),
        Ty::Var(tv) => p.tparam(*tv).map(|tp| tp.tags.contains(&tag)).unwrap_or(false),
        _ => true,
    }
}

pub fn class_carries(p: &mut Program, id: ClassId, tag: ClassId) -> bool {
    let uses = p.class(id).ann_uses.clone();
    let unit = p.class(id).unit;
    for a in &uses {
        if lookup_class(p, unit, &a.name, a.pos, false) == Some(tag) {
            return true;
        }
    }
    if !p.class(tag).inherited {
        return false;
    }
    for sup in p.direct_supers(id) {
        if let Some(sid) = sup.class_id() {
            if sid != id && class_carries(p, sid, tag) {
                return true;
            }
        }
    }
    false
}

// ---- annotation kinds ----

fn qualifier_kinds(p: &mut Program) {
    for id in 0..p.classes.len() as ClassId {
        if !p.class(id).is_annotation() {
            continue;
        }
        let unit = p.class(id).unit;
        let uses = p.class(id).ann_uses.clone();
        let mut kind = Qualifier::No;
        for a in &uses {
            let Some(aid) = lookup_class(p, unit, &a.name, a.pos, false) else { continue };
            if aid == p.wk.refines || aid == p.wk.widens {
                if kind != Qualifier::No {
                    p.error(unit, a.pos, "a qualifier is declared with exactly one of `@Refines` and `@Widens`");
                }
                kind = if aid == p.wk.refines { Qualifier::Refines } else { Qualifier::Widens };
            }
            if aid == p.wk.inherited {
                p.classes[id as usize].inherited = true;
            }
        }
        p.classes[id as usize].qualifier = kind;
    }
}

// ---- headers ----

fn headers(p: &mut Program) {
    let n = p.classes.len() as ClassId;
    // Names and variance first, so that bounds may mention any parameter.
    for id in 0..n {
        let d = decl_of(p, id);
        let mut seen = HashSet::new();
        for tp in &d.type_params {
            if !seen.insert(tp.name.clone()) {
                p.error(p.class(id).unit, tp.pos, format!("the type parameter `{}` is declared twice", tp.name));
            }
            if tp.variance != Variance::Invariant && d.kind != TypeKind::Class && d.kind != TypeKind::Interface && d.kind != TypeKind::ValueClass {
                p.error(p.class(id).unit, tp.pos, "`in` and `out` are written on a class or an interface");
            }
            p.classes[id as usize].tparams.push(TParam {
                name: tp.name.clone(),
                variance: tp.variance,
                bounds: Vec::new(),
                tags: Vec::new(),
                pos: tp.pos,
            });
        }
    }
    for id in 0..n {
        let d = decl_of(p, id);
        let unit = p.class(id).unit;
        let cx = TypeCx::for_class(p, id, true);
        for (i, tp) in d.type_params.iter().enumerate() {
            let (bounds, tags) = resolve_tparam(p, &cx, tp);
            p.classes[id as usize].tparams[i].bounds = bounds;
            p.classes[id as usize].tparams[i].tags = tags;
        }
        let wk = p.wk.clone();
        // The superclass.
        let mut superclass = None;
        match d.kind {
            TypeKind::Class => {
                if let Some(tr) = d.extends.first() {
                    let t = resolve_type(p, &cx, tr);
                    if let Some(sid) = t.class_id() {
                        let s = p.class(sid).clone();
                        if sid == wk.enum_ {
                            p.error(unit, tr.pos, "a class does not extend `Enum`; an enum is declared with `enum`");
                        } else if s.kind != TypeKind::Class {
                            p.error(unit, tr.pos, format!("`{}` is not a class that can be extended", s.name));
                        } else if s.is_final() {
                            p.error(unit, tr.pos, format!("`{}` is final; only an `open`, `sealed` or `abstract` class is extended", s.name));
                        }
                        if t.has_wildcard_arg() {
                            p.error(unit, tr.pos, "a wildcard is not written for the arguments of a superclass");
                        }
                        if !t.quals.is_empty() || t.nullable {
                            p.error(unit, tr.pos, "a qualifier is not written on a supertype");
                        }
                    }
                    superclass = Some(t.bare());
                } else if id != wk.object {
                    superclass = Some(Type::simple(wk.object));
                }
            }
            TypeKind::ValueClass => {
                if let Some(tr) = d.extends.first() {
                    p.error(unit, tr.pos, "a value class extends no class");
                }
                if id != wk.null {
                    superclass = Some(Type::simple(wk.object));
                }
            }
            TypeKind::Enum => superclass = Some(Type::simple(wk.enum_)),
            TypeKind::Annotation => superclass = Some(Type::simple(wk.object)),
            TypeKind::Interface => {}
        }
        p.classes[id as usize].superclass = superclass;
        // The interfaces.
        let written: &[ast::TypeRef] = if d.kind == TypeKind::Interface { &d.extends } else { &d.implements };
        let mut interfaces = Vec::new();
        for tr in written {
            let t = resolve_type(p, &cx, tr);
            if let Some(sid) = t.class_id() {
                if sid == wk.annotation {
                    p.error(unit, tr.pos, "only an annotation implements `Annotation`");
                } else if !p.class(sid).is_interface() {
                    let name = p.class(sid).name.clone();
                    p.error(unit, tr.pos, format!("`{name}` is not an interface"));
                    continue;
                }
                if t.has_wildcard_arg() {
                    p.error(unit, tr.pos, "a wildcard is not written for the arguments of a supertype");
                }
                if interfaces.iter().any(|o: &Type| o.class_id() == Some(sid)) {
                    p.error(unit, tr.pos, "an interface is named once in a declaration");
                    continue;
                }
                interfaces.push(t.bare());
            }
        }
        if d.kind == TypeKind::Annotation {
            interfaces.push(Type::simple(wk.annotation));
        }
        p.classes[id as usize].interfaces = interfaces;
        if let Some(list) = &d.permits {
            let mut permits = Vec::new();
            for tr in list {
                if let Some(pid) = lookup_class(p, unit, &tr.name, tr.pos, true) {
                    permits.push(pid);
                }
            }
            p.classes[id as usize].permits = Some(permits);
        }
    }
    // Checks that need every header.
    for id in 0..n {
        let unit = p.class(id).unit;
        let pos = p.class(id).pos;
        // A cycle.
        let mut seen = vec![id];
        let mut cur = p.class(id).superclass.clone();
        while let Some(t) = cur {
            let Some(sid) = t.class_id() else { break };
            if seen.contains(&sid) {
                p.error(unit, pos, "this class extends itself");
                p.classes[id as usize].superclass = Some(Type::simple(p.wk.object));
                break;
            }
            seen.push(sid);
            cur = p.class(sid).superclass.clone();
        }
        // Sealed supertypes must permit this type, and permitted types must extend.
        for sup in p.direct_supers(id) {
            let Some(sid) = sup.class_id() else { continue };
            if let Some(permits) = p.class(sid).permits.clone() {
                if p.class(sid).is_sealed && !permits.contains(&id) {
                    let name = p.class(sid).name.clone();
                    p.error(unit, pos, format!("`{name}` is sealed and does not permit this type"));
                }
            }
            if !type_visible(p, sid, unit) {
                let name = p.class(sid).name.clone();
                p.error(unit, pos, format!("`{name}` is not visible here"));
            }
        }
        if let Some(permits) = p.class(id).permits.clone() {
            for pid in permits {
                let extends = p.direct_supers(pid).iter().any(|s| s.class_id() == Some(id));
                if !extends {
                    let name = p.class(pid).name.clone();
                    p.error(unit, pos, format!("`{name}` is permitted and does not extend or implement this type"));
                }
            }
        }
        // One set of type arguments for each generic interface.
        let mut all: Vec<Type> = Vec::new();
        let mut work = p.direct_supers(id);
        while let Some(t) = work.pop() {
            let Ty::Class(sid, sargs) = &t.ty else { continue };
            if let Some(other) = all.iter().find(|o| o.class_id() == Some(*sid)) {
                if other != &t {
                    let name = p.class(*sid).name.clone();
                    p.error(unit, pos, format!("`{name}` is implemented with two sets of type arguments"));
                }
                continue;
            }
            let s = Subst::for_class(*sid, sargs);
            for up in p.direct_supers(*sid) {
                work.push(s.apply(&up));
            }
            all.push(t);
        }
    }
}

fn decl_of(p: &Program, id: ClassId) -> ast::TypeDecl {
    let c = p.class(id);
    p.units[c.unit].types[c.decl_index].clone()
}

fn resolve_tparam(p: &mut Program, cx: &TypeCx, tp: &ast::TypeParam) -> (Vec<Type>, Vec<ClassId>) {
    let mut bounds = Vec::new();
    let mut classes = 0;
    for (i, b) in tp.bounds.iter().enumerate() {
        let t = resolve_type(p, cx, b);
        if let Some(bid) = t.class_id() {
            if !p.class(bid).is_interface() {
                classes += 1;
                if classes > 1 {
                    p.error(cx.unit, b.pos, "at most one bound is a class; the others are interfaces");
                }
            }
        } else if matches!(t.ty, Ty::Var(_)) && (i > 0 || tp.bounds.len() > 1) {
            p.error(cx.unit, b.pos, "a type parameter is a bound only on its own");
        }
        bounds.push(t);
    }
    let mut tags = Vec::new();
    for a in &tp.annotations {
        let Some(aid) = lookup_class(p, cx.unit, &a.name, a.pos, true) else { continue };
        if !p.class(aid).is_annotation() || p.class(aid).qualifier != Qualifier::No {
            p.error(cx.unit, a.pos, "a type parameter requires a tag: an annotation that is written on type declarations");
            continue;
        }
        if !matches!(a.args, ast::AnnArgs::None) {
            p.error(cx.unit, a.pos, "a requirement names the annotation and no arguments");
        }
        tags.push(aid);
    }
    (bounds, tags)
}

// ---- members ----

fn member_aud(mods: &ast::Mods, default: Aud) -> Aud {
    match mods.audience {
        None => default,
        Some(Audience::Private) => Aud::Private,
        Some(Audience::Package) => Aud::Package,
        Some(Audience::Protected) => Aud::Protected,
        Some(Audience::Public) => Aud::Public,
    }
}

fn only_list(p: &mut Program, cx: &TypeCx, mods: &ast::Mods, owner: ClassId) -> Option<Vec<ClassId>> {
    let list = mods.only.as_ref()?;
    let mut out = Vec::new();
    for tr in list {
        let Some(id) = lookup_class(p, cx.unit, &tr.name, tr.pos, true) else { continue };
        if out.contains(&id) {
            p.error(cx.unit, tr.pos, "a type is listed once in `only`");
            continue;
        }
        match mods.audience {
            Some(Audience::Package) if p.class(id).package != p.class(owner).package => {
                p.error(cx.unit, tr.pos, "after `package`, `only` lists types of the same package");
            }
            Some(Audience::Protected) if !p.is_subclass(id, owner) => {
                p.error(cx.unit, tr.pos, "after `protected`, `only` lists subclasses of the declaring type");
            }
            _ => {}
        }
        out.push(id);
    }
    Some(out)
}

fn has_ann(p: &mut Program, unit: usize, uses: &[ast::AnnotationUse], id: ClassId) -> bool {
    uses.iter().any(|a| lookup_class(p, unit, &a.name, a.pos, false) == Some(id))
}

fn make_params(p: &mut Program, cx: &TypeCx, params: &[ast::Param]) -> Vec<Param> {
    let mut out: Vec<Param> = Vec::new();
    for (i, prm) in params.iter().enumerate() {
        let (mut ty, anns) = resolve_type_anns(p, cx, &prm.ty, &prm.mods.annotations, true);
        if prm.varargs {
            if i + 1 != params.len() {
                p.error(cx.unit, prm.pos, "a varargs parameter is the last parameter");
            }
            ty = p.array_of(ty);
        }
        let m = &prm.mods;
        if m.audience.is_some() || m.is_static || m.is_open || m.is_abstract || m.is_sealed || m.is_foreign {
            p.error(cx.unit, prm.pos, "a parameter accepts `final` and annotations only");
        }
        if out.iter().any(|o| o.name == prm.name) {
            p.error(cx.unit, prm.pos, format!("the parameter `{}` is declared twice", prm.name));
        }
        out.push(Param {
            name: prm.name.clone(),
            ty,
            varargs: prm.varargs,
            is_final: m.is_final,
            ann_uses: anns,
            anns: Vec::new(),
            pos: prm.pos,
        });
    }
    out
}

fn members(p: &mut Program) {
    let n = p.classes.len() as ClassId;
    for id in 0..n {
        let d = decl_of(p, id);
        let unit = p.class(id).unit;
        let kind = d.kind;
        let is_iface = kind == TypeKind::Interface;
        let cx_inst = TypeCx::for_class(p, id, true);
        let cx_static = TypeCx::for_class(p, id, false);
        let self_type = {
            let c = p.class(id);
            let args = (0..c.tparams.len())
                .map(|i| Arg::Ty(Type::var(Tv { owner: TvOwner::Class(id), index: i as u32 })))
                .collect();
            Type::class(id, args)
        };

        // Enum constants are public static final fields.
        for (i, k) in d.constants.iter().enumerate() {
            if p.class(id).fields.iter().any(|f| f.name == k.name) {
                p.error(unit, k.pos, format!("the constant `{}` is declared twice", k.name));
            }
            p.classes[id as usize].fields.push(Field {
                name: k.name.clone(),
                ty: Type::simple(id),
                is_static: true,
                is_final: true,
                aud: Aud::Public,
                only: None,
                init: None,
                ann_uses: k.annotations.clone(),
                anns: Vec::new(),
                enum_ordinal: Some(i as u32),
                enum_args: k.args.clone(),
                constant: None,
                pos: k.pos,
            });
        }
        // Annotation elements are public fields.
        for e in &d.elements {
            let ty = resolve_type(p, &cx_static, &e.ty);
            if !element_type_ok(p, &ty) {
                p.error(unit, e.pos, "an element is a numeric class, Boolean, Char, String, an enum, Class, an annotation, or an array of one of those");
            }
            if p.class(id).qualifier != Qualifier::No {
                p.error(unit, e.pos, "a qualifier has no elements");
            }
            p.classes[id as usize].elements.push(Element {
                name: e.name.clone(),
                ty: ty.clone(),
                default_src: e.default.clone(),
                default: None,
                pos: e.pos,
            });
            p.classes[id as usize].fields.push(Field {
                name: e.name.clone(),
                ty,
                is_static: false,
                is_final: true,
                aud: Aud::Public,
                only: None,
                init: None,
                ann_uses: Vec::new(),
                anns: Vec::new(),
                enum_ordinal: None,
                enum_args: Vec::new(),
                constant: None,
                pos: e.pos,
            });
        }

        let mut compact_seen = false;
        for m in &d.members {
            match m {
                Member::Field(f) => {
                    let mods = &f.mods;
                    let is_static = mods.is_static || is_iface;
                    let cx = if is_static { &cx_static } else { &cx_inst };
                    let (ty, anns) = resolve_type_anns(p, cx, &f.ty, &mods.annotations, true);
                    if mods.is_open || mods.is_abstract || mods.is_sealed || mods.is_foreign {
                        p.error(unit, f.pos, "a field accepts an audience, `static` and `final`");
                    }
                    if p.class(id).fields.iter().any(|o| o.name == f.name) {
                        p.error(unit, f.pos, format!("the field `{}` is declared twice", f.name));
                    }
                    let mut aud = member_aud(mods, Aud::Private);
                    if is_iface {
                        if matches!(mods.audience, Some(a) if a != Audience::Public) {
                            p.error(unit, f.pos, "a field of an interface is public");
                        }
                        aud = Aud::Public;
                        if f.init.is_none() {
                            p.error(unit, f.pos, "a field of an interface is a constant and needs an initializer");
                        }
                    }
                    if kind == TypeKind::ValueClass && !is_static && f.init.is_some() {
                        p.error(unit, f.pos, "a field of a value class has no initializer");
                    }
                    if kind == TypeKind::Annotation {
                        p.error(unit, f.pos, "an annotation declares its elements in its header");
                    }
                    let only = only_list(p, cx, mods, id);
                    let value_field = p.class(id).is_value() && !is_static;
                    p.classes[id as usize].fields.push(Field {
                        name: f.name.clone(),
                        ty,
                        is_static,
                        is_final: mods.is_final || is_iface || value_field,
                        aud,
                        only,
                        init: f.init.clone(),
                        ann_uses: anns,
                        anns: Vec::new(),
                        enum_ordinal: None,
                        enum_args: Vec::new(),
                        constant: None,
                        pos: f.pos,
                    });
                }
                Member::Method(md) => {
                    let mods = &md.mods;
                    let index = p.class(id).methods.len() as u32;
                    let mref = MethodRef { class: id, index };
                    // A static requirement of an interface may use the interface's type
                    // parameters, which stand for the implementer's arguments.
                    let requirement = is_iface && mods.is_static && md.body.is_none();
                    let mut cx = if mods.is_static && !requirement { cx_static.clone() } else { cx_inst.clone() };
                    let mut seen = HashSet::new();
                    for (i, tp) in md.type_params.iter().enumerate() {
                        if !seen.insert(tp.name.clone()) || cx.tvars.iter().any(|(n, _)| *n == tp.name) {
                            p.error(unit, tp.pos, format!("the type parameter `{}` has the name of another one in scope", tp.name));
                        }
                        if tp.variance != Variance::Invariant {
                            p.error(unit, tp.pos, "the type parameters of a method have no variance");
                        }
                        cx.tvars.push((tp.name.clone(), Tv { owner: TvOwner::Method(mref), index: i as u32 }));
                    }
                    // The method must exist before its bounds are resolved, because a
                    // bound may mention the method's own parameters.
                    let has_override = has_ann(p, unit, &mods.annotations, p.wk.override_);
                    let discardable = has_ann(p, unit, &mods.annotations, p.wk.discardable);
                    let implicit = has_ann(p, unit, &mods.annotations, p.wk.implicit);
                    let intrinsic = has_ann(p, unit, &mods.annotations, p.wk.intrinsic);
                    let has_body = md.body.is_some();
                    let static_requirement = is_iface && mods.is_static && !has_body;
                    let is_abstract = mods.is_abstract || (is_iface && !mods.is_static && !has_body);
                    p.classes[id as usize].methods.push(Method {
                        name: md.name.clone(),
                        tparams: md
                            .type_params
                            .iter()
                            .map(|tp| TParam { name: tp.name.clone(), variance: Variance::Invariant, bounds: Vec::new(), tags: Vec::new(), pos: tp.pos })
                            .collect(),
                        params: Vec::new(),
                        ret: Type::error(),
                        recv_nullable: false,
                        recv_quals: Vec::new(),
                        is_static: mods.is_static,
                        is_open: mods.is_open,
                        is_abstract,
                        is_final_written: mods.is_final,
                        foreign: mods.is_foreign,
                        intrinsic,
                        discardable,
                        has_override,
                        implicit,
                        narrows: None,
                        symbol: None,
                        deprecated: None,
                        aud: Aud::Private,
                        only: None,
                        body: md.body.clone(),
                        ann_uses: Vec::new(),
                        anns: Vec::new(),
                        overrides: Vec::new(),
                        static_requirement,
                        pos: md.pos,
                        checked: None,
                    });
                    for (i, tp) in md.type_params.iter().enumerate() {
                        let (bounds, tags) = resolve_tparam(p, &cx, tp);
                        let m = &mut p.classes[id as usize].methods[index as usize];
                        m.tparams[i].bounds = bounds;
                        m.tparams[i].tags = tags;
                    }
                    let (ret, anns) = match &md.result {
                        Some(tr) => resolve_type_anns(p, &cx, tr, &mods.annotations, true),
                        None => {
                            let mut unit_ty = Type::simple(p.wk.unit);
                            let rest = apply_annotations(p, &cx, &mut unit_ty, &mods.annotations, true);
                            if unit_ty.nullable || !unit_ty.quals.is_empty() {
                                p.error(unit, md.pos, "a qualifier among a method's modifiers qualifies its result, and this method has none");
                            }
                            (Type::simple(p.wk.unit), rest)
                        }
                    };
                    let params = make_params(p, &cx, &md.params);
                    let mut recv_nullable = false;
                    let mut recv_quals = Vec::new();
                    if let Some(rt) = &md.receiver {
                        if mods.is_static {
                            p.error(unit, rt.pos, "a static method does not declare a receiver");
                        }
                        let t = resolve_type(p, &cx_inst, rt);
                        if !t.is_error() && t.bare() != self_type {
                            p.error(unit, rt.pos, "the receiver parameter has the type of the declaring class");
                        }
                        recv_nullable = t.nullable;
                        recv_quals = t.quals.clone();
                    }
                    let mut aud = member_aud(mods, Aud::Private);
                    if is_iface {
                        if matches!(mods.audience, Some(a) if a != Audience::Public) {
                            p.error(unit, md.pos, "a method of an interface is public");
                        }
                        aud = Aud::Public;
                    }
                    let only = only_list(p, &cx, mods, id);
                    // Modifier rules of sections 4.1 and 2.6.
                    if mods.is_sealed {
                        p.error(unit, md.pos, "`sealed` is written on a type");
                    }
                    if mods.is_open && mods.is_final {
                        p.error(unit, md.pos, "`open final` is rejected");
                    }
                    if mods.is_abstract && (mods.is_final || mods.is_static) {
                        p.error(unit, md.pos, "`abstract final` and `abstract static` are rejected");
                    }
                    if mods.is_open && mods.is_static {
                        p.error(unit, md.pos, "a static method is not overridden, so `open` is rejected on it");
                    }
                    if mods.is_abstract && !p.class(id).is_abstract && !is_iface {
                        p.error(unit, md.pos, "an abstract method appears only in an abstract class");
                    }
                    if mods.is_foreign && !mods.is_static {
                        p.error(unit, md.pos, "a `foreign` method is also declared `static`");
                    }
                    if (mods.is_open || mods.is_abstract) && p.class(id).is_value() {
                        p.error(unit, md.pos, "a value class is final, so its methods are neither `open` nor `abstract`");
                    }
                    if intrinsic && p.class(id).package != "cleat" {
                        p.error(unit, md.pos, "`@Intrinsic` is rejected outside the package `cleat`");
                    }
                    let bodiless_ok = is_abstract || mods.is_foreign || intrinsic || static_requirement;
                    if !has_body && !bodiless_ok {
                        p.error(unit, md.pos, format!("the method `{}` needs a body", md.name));
                    }
                    if has_body && (mods.is_abstract || mods.is_foreign || intrinsic) {
                        p.error(unit, md.pos, "an abstract, `foreign` or `@Intrinsic` method has no body");
                    }
                    if is_iface && mods.is_final && !has_body {
                        p.error(unit, md.pos, "`final` on an interface method without a body is rejected");
                    }
                    if recv_nullable && (mods.is_open || is_abstract) && id != p.wk.object {
                        p.error(unit, md.pos, "an `open` or `abstract` method does not declare a `@Nullable` receiver");
                    }
                    let m = &mut p.classes[id as usize].methods[index as usize];
                    m.params = params;
                    m.ret = ret;
                    m.recv_nullable = recv_nullable;
                    m.recv_quals = recv_quals;
                    m.aud = aud;
                    m.only = only;
                    m.ann_uses = anns;
                }
                Member::Ctor(cd) => {
                    let mods = &cd.mods;
                    if mods.is_static || mods.is_final || mods.is_open || mods.is_abstract || mods.is_sealed || mods.is_foreign {
                        p.error(unit, cd.pos, "a constructor accepts an audience and annotations only");
                    }
                    let intrinsic = has_ann(p, unit, &mods.annotations, p.wk.intrinsic);
                    let mut dummy = Type::simple(p.wk.unit);
                    let anns = apply_annotations(p, &cx_inst, &mut dummy, &mods.annotations, true);
                    let aud = member_aud(mods, Aud::Private);
                    let only = only_list(p, &cx_inst, mods, id);
                    match (&cd.params, kind) {
                        (_, TypeKind::Interface) | (_, TypeKind::Annotation) => {
                            p.error(unit, cd.pos, "an interface and an annotation declare no constructor");
                        }
                        (None, TypeKind::ValueClass) => {
                            if compact_seen {
                                p.error(unit, cd.pos, "a value class has exactly one constructor");
                            }
                            compact_seen = true;
                            p.classes[id as usize].ctors.push(Ctor {
                                params: Vec::new(),
                                aud,
                                only,
                                body: cd.body.clone(),
                                of_value: true,
                                implicit: false,
                                intrinsic,
                                ann_uses: anns,
                                anns: Vec::new(),
                                deprecated: None,
                                pos: cd.pos,
                                checked: None,
                            });
                        }
                        (None, _) => {
                            p.error(unit, cd.pos, "only a value class writes its constructor in the compact form");
                        }
                        (Some(_), TypeKind::ValueClass) if !intrinsic => {
                            p.error(unit, cd.pos, "a value class has one constructor, whose parameters are its fields; it may be written in the compact form `Name { ... }`");
                        }
                        (Some(params), _) => {
                            if kind == TypeKind::Enum && matches!(mods.audience, Some(a) if a != Audience::Private) {
                                p.error(unit, cd.pos, "the constructors of an enum are private");
                            }
                            if intrinsic && p.class(id).package != "cleat" {
                                p.error(unit, cd.pos, "`@Intrinsic` is rejected outside the package `cleat`");
                            }
                            if cd.body.is_none() && !intrinsic {
                                p.error(unit, cd.pos, "a constructor needs a body");
                            }
                            let params = make_params(p, &cx_inst, params);
                            p.classes[id as usize].ctors.push(Ctor {
                                params,
                                aud,
                                only,
                                body: cd.body.clone(),
                                of_value: false,
                                implicit: false,
                                intrinsic,
                                ann_uses: anns,
                                anns: Vec::new(),
                                deprecated: None,
                                pos: cd.pos,
                                checked: None,
                            });
                        }
                    }
                }
                Member::StaticInit(b) => {
                    if is_iface || kind == TypeKind::Annotation {
                        p.error(unit, b.pos, "a static initializer block belongs to a class");
                    }
                    p.classes[id as usize].static_inits.push(b.clone());
                }
            }
        }

        // The constructor a class has when it writes none.
        let pos = p.class(id).pos;
        match kind {
            TypeKind::ValueClass | TypeKind::Annotation => {
                let params: Vec<Param> = p
                    .class(id)
                    .fields
                    .iter()
                    .filter(|f| !f.is_static)
                    .map(|f| Param {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                        varargs: false,
                        is_final: true,
                        ann_uses: Vec::new(),
                        anns: Vec::new(),
                        pos: f.pos,
                    })
                    .collect();
                let c = &mut p.classes[id as usize];
                let compact = c.ctors.iter().position(|k| k.of_value);
                match compact {
                    Some(k) => c.ctors[k].params = params,
                    None if c.ctors.is_empty() => c.ctors.push(Ctor {
                        params,
                        // An annotation is constructed only by its uses.
                        aud: if kind == TypeKind::Annotation { Aud::Private } else { Aud::Public },
                        only: None,
                        body: None,
                        of_value: true,
                        implicit: true,
                        intrinsic: false,
                        ann_uses: Vec::new(),
                        anns: Vec::new(),
                        deprecated: None,
                        pos,
                        checked: None,
                    }),
                    None => {}
                }
            }
            TypeKind::Class | TypeKind::Enum => {
                if p.class(id).ctors.is_empty() {
                    p.classes[id as usize].ctors.push(Ctor {
                        params: Vec::new(),
                        aud: if kind == TypeKind::Enum { Aud::Private } else { Aud::Public },
                        only: None,
                        body: None,
                        of_value: false,
                        implicit: true,
                        intrinsic: false,
                        ann_uses: Vec::new(),
                        anns: Vec::new(),
                        deprecated: None,
                        pos,
                        checked: None,
                    });
                }
            }
            TypeKind::Interface => {}
        }
    }
}

fn element_type_ok(p: &Program, t: &Type) -> bool {
    if t.nullable || !t.quals.is_empty() {
        return false;
    }
    let inner = p.array_elem(t).unwrap_or_else(|| t.clone());
    if inner.nullable || !inner.quals.is_empty() {
        return false;
    }
    let Some(id) = inner.class_id() else { return inner.is_error() };
    let w = &p.wk;
    p.num_kind(id).is_some()
        || id == w.boolean
        || id == w.char_
        || id == w.string
        || id == w.class
        || p.class(id).is_enum()
        || p.class(id).is_annotation()
}

// ---- the hierarchy: overriding and completeness ----

/// Every proper supertype of a class, instantiated over the class's own parameters.
pub fn all_supertypes(p: &mut Program, id: ClassId) -> Vec<Type> {
    let mut all: Vec<Type> = Vec::new();
    let mut work = p.direct_supers(id);
    work.reverse();
    while let Some(t) = work.pop() {
        let Ty::Class(sid, sargs) = &t.ty else { continue };
        if *sid == id || all.iter().any(|o| o.class_id() == Some(*sid)) {
            continue;
        }
        let s = Subst::for_class(*sid, sargs);
        let mut ups: Vec<Type> = p.direct_supers(*sid).iter().map(|u| s.apply(u)).collect();
        ups.reverse();
        all.push(t);
        work.extend(ups);
    }
    all
}

/// Whether the parameter lists of two methods are the same once `n`'s types are seen
/// from `m`'s class. Returns the substitution that maps `n`'s signature there.
pub fn same_signature(p: &mut Program, m: MethodRef, n: MethodRef, n_class_subst: &Subst) -> Option<Subst> {
    let (mm, nn) = (p.method(m).clone(), p.method(n).clone());
    if mm.name != nn.name || mm.params.len() != nn.params.len() || mm.tparams.len() != nn.tparams.len() {
        return None;
    }
    let mut s = n_class_subst.clone();
    for i in 0..nn.tparams.len() {
        s.bind(
            Tv { owner: TvOwner::Method(n), index: i as u32 },
            Type::var(Tv { owner: TvOwner::Method(m), index: i as u32 }),
        );
    }
    for (a, b) in mm.params.iter().zip(nn.params.iter()) {
        let bt = s.apply(&b.ty);
        if a.ty != bt && !(p.same_type(&a.ty, &bt)) {
            return None;
        }
    }
    Some(s)
}

fn aud_rank(a: Aud) -> u32 {
    match a {
        Aud::Private => 0,
        Aud::File | Aud::Package => 1,
        Aud::Protected => 2,
        Aud::Public => 3,
    }
}

fn hierarchy(p: &mut Program) {
    let n = p.classes.len() as ClassId;
    let special = ["equals", "hashCode", "toString"];
    for id in 0..n {
        let unit = p.class(id).unit;
        let supers = all_supertypes(p, id);
        let is_iface = p.class(id).is_interface();
        for mi in 0..p.class(id).methods.len() as u32 {
            let m = MethodRef { class: id, index: mi };
            let mm = p.method(m).clone();
            let mut overrides = Vec::new();
            for sup in &supers {
                let Ty::Class(sid, sargs) = &sup.ty else { continue };
                let subst = Subst::for_class(*sid, sargs);
                let sup_is_iface = p.class(*sid).is_interface();
                for ni in 0..p.class(*sid).methods.len() as u32 {
                    let nref = MethodRef { class: *sid, index: ni };
                    let nn = p.method(nref).clone();
                    if nn.name != mm.name || nn.aud == Aud::Private {
                        continue;
                    }
                    let Some(s) = same_signature(p, m, nref, &subst) else { continue };
                    if mm.is_static != nn.is_static {
                        p.error(unit, mm.pos, format!("`{}` is static in one declaration and not in the other", mm.name));
                        continue;
                    }
                    if nn.is_static && !nn.static_requirement {
                        // A static method hides; it overrides nothing.
                        continue;
                    }
                    if !nn.is_static && !nn.is_virtual(sup_is_iface) {
                        let sname = p.class(*sid).name.clone();
                        p.error(unit, mm.pos, format!("`{}` is final in `{sname}` and cannot be overridden", mm.name));
                        continue;
                    }
                    // Result: the same type or a subtype.
                    let want = s.apply(&nn.ret);
                    if !p.is_subtype(&mm.ret, &want) {
                        let (a, b) = (p.show(&mm.ret), p.show(&want));
                        p.error(unit, mm.pos, format!("the result `{a}` is not `{b}` or a subtype of it, which the overridden method returns"));
                    }
                    if aud_rank(mm.aud) < aud_rank(nn.aud) {
                        p.error(unit, mm.pos, "an overriding method has the audience of the method it overrides, or a wider one");
                    }
                    if sup_is_iface && mm.aud != Aud::Public {
                        p.error(unit, mm.pos, "a method that implements an interface method is `public`");
                    }
                    if mm.discardable != nn.discardable {
                        p.error(unit, mm.pos, "a method carries `@Discardable` exactly when the method it overrides does");
                    }
                    if !special.contains(&mm.name.as_str()) && (mm.recv_nullable != nn.recv_nullable || mm.recv_quals != nn.recv_quals) {
                        p.error(unit, mm.pos, "an override has the same receiver qualifiers as the method it overrides");
                    }
                    overrides.push(nref);
                }
            }
            if !overrides.is_empty() && !mm.has_override {
                p.error(unit, mm.pos, format!("`{}` overrides or implements a method and must carry `@Override`", mm.name));
            }
            if overrides.is_empty() && mm.has_override {
                p.error(unit, mm.pos, format!("`{}` carries `@Override` and overrides nothing", mm.name));
            }
            p.classes[id as usize].methods[mi as usize].overrides = overrides;
        }
        // A class with instances has a body for every method it must have.
        let c = p.class(id).clone();
        if !is_iface && !c.is_abstract && !c.is_annotation() {
            for sup in &supers {
                let Ty::Class(sid, _) = &sup.ty else { continue };
                for ni in 0..p.class(*sid).methods.len() as u32 {
                    let nref = MethodRef { class: *sid, index: ni };
                    let nn = p.method(nref).clone();
                    if nn.static_requirement {
                        if find_static_impl(p, id, nref).is_none() {
                            let sname = p.class(*sid).name.clone();
                            p.error(unit, c.pos, format!("`{}` does not declare the static method `{}` that `{sname}` requires", c.name, nn.name));
                        }
                    } else if nn.is_abstract {
                        match impl_of(p, id, nref) {
                            ImplOf::Found(f) if !p.method(f).is_abstract => {}
                            ImplOf::Conflict => {}
                            _ => {
                                let sname = p.class(*sid).name.clone();
                                p.error(unit, c.pos, format!("`{}` does not implement `{}` of `{sname}`", c.name, nn.name));
                            }
                        }
                    }
                }
            }
        }
        // Two interface bodies for one signature must be settled by the class.
        if !is_iface {
            for sup in &supers {
                let Ty::Class(sid, _) = &sup.ty else { continue };
                if !p.class(*sid).is_interface() {
                    continue;
                }
                for ni in 0..p.class(*sid).methods.len() as u32 {
                    let nref = MethodRef { class: *sid, index: ni };
                    if p.method(nref).is_static {
                        continue;
                    }
                    if let ImplOf::Conflict = impl_of(p, id, nref) {
                        let name = p.method(nref).name.clone();
                        p.error(unit, c.pos, format!("two interfaces provide a body for `{name}`; the class must override it"));
                    }
                }
            }
        }
        variance_checks(p, id);
    }
}

pub enum ImplOf {
    Found(MethodRef),
    None,
    Conflict,
}

/// The method that a class has for the signature of `root`.
pub fn impl_of(p: &mut Program, class: ClassId, root: MethodRef) -> ImplOf {
    // The class chain first: a method declared in a class wins over an interface body.
    let mut cur = Some(class);
    while let Some(cid) = cur {
        if !p.class(cid).is_interface() || cid == class {
            for (i, m) in p.class(cid).methods.iter().enumerate() {
                let mref = MethodRef { class: cid, index: i as u32 };
                if mref == root || m.overrides.contains(&root) {
                    if !(p.class(cid).is_interface() && m.is_abstract) {
                        return ImplOf::Found(mref);
                    }
                }
            }
        }
        cur = p.class(cid).superclass.as_ref().and_then(|t| t.class_id());
    }
    // Then the bodies that interfaces provide.
    let mut found: Vec<MethodRef> = Vec::new();
    for sup in all_supertypes(p, class) {
        let Some(sid) = sup.class_id() else { continue };
        if !p.class(sid).is_interface() {
            continue;
        }
        for (i, m) in p.class(sid).methods.iter().enumerate() {
            let mref = MethodRef { class: sid, index: i as u32 };
            if (mref == root || m.overrides.contains(&root)) && !m.is_abstract && !m.is_static {
                found.push(mref);
            }
        }
    }
    // Keep the most specific: drop a body that another found body overrides.
    let all = found.clone();
    found.retain(|f| !all.iter().any(|g| g != f && p.method(*g).overrides.contains(f)));
    match found.len() {
        0 => {
            if root.class == class || p.is_subclass(class, root.class) {
                // The declaration itself, when nothing gives it a body.
                ImplOf::Found(root)
            } else {
                ImplOf::None
            }
        }
        1 => ImplOf::Found(found[0]),
        _ => ImplOf::Conflict,
    }
}

/// The public static method of a class, declared or inherited, that meets a requirement.
pub fn find_static_impl(p: &mut Program, class: ClassId, req: MethodRef) -> Option<MethodRef> {
    let mut cur = Some(class);
    while let Some(cid) = cur {
        for (i, m) in p.class(cid).methods.iter().enumerate() {
            if m.is_static && !m.static_requirement && m.overrides.contains(&req) {
                return Some(MethodRef { class: cid, index: i as u32 });
            }
        }
        cur = p.class(cid).superclass.as_ref().and_then(|t| t.class_id());
    }
    None
}

/// Section 7.3: an `out` parameter stands only where values are produced, and an `in`
/// parameter only where they are consumed.
fn variance_checks(p: &mut Program, id: ClassId) {
    let c = p.class(id).clone();
    if c.tparams.iter().all(|t| t.variance == Variance::Invariant) {
        return;
    }
    let mut bad: Vec<(Pos, String)> = Vec::new();
    for m in &c.methods {
        if m.is_static {
            continue;
        }
        for prm in &m.params {
            position(p, id, &prm.ty, -1, prm.pos, &mut bad);
        }
        position(p, id, &m.ret, 1, m.pos, &mut bad);
    }
    for f in &c.fields {
        if !f.is_static {
            position(p, id, &f.ty, if f.is_final { 1 } else { 0 }, f.pos, &mut bad);
        }
    }
    for sup in p.direct_supers(id) {
        position(p, id, &sup, 1, c.pos, &mut bad);
    }
    for (pos, msg) in bad {
        p.error(c.unit, pos, msg);
    }
}

/// `dir` is 1 where values are produced, -1 where they are consumed, 0 where both.
fn position(p: &Program, owner: ClassId, t: &Type, dir: i32, pos: Pos, bad: &mut Vec<(Pos, String)>) {
    match &t.ty {
        Ty::Var(tv) if tv.owner == TvOwner::Class(owner) => {
            let tp = &p.class(owner).tparams[tv.index as usize];
            let ok = match tp.variance {
                Variance::Invariant => true,
                Variance::Out => dir == 1,
                Variance::In => dir == -1,
            };
            if !ok {
                let word = if tp.variance == Variance::Out { "out" } else { "in" };
                let place = match dir {
                    1 => "where a value is produced",
                    -1 => "where a value is consumed",
                    _ => "where a value is both produced and consumed",
                };
                bad.push((pos, format!("`{}` is declared `{word}` and is written {place}", tp.name)));
            }
        }
        Ty::Class(cid, args) => {
            for (i, a) in args.iter().enumerate() {
                let v = p.class(*cid).tparams.get(i).map(|t| t.variance).unwrap_or(Variance::Invariant);
                let sub = match v {
                    Variance::Out => dir,
                    Variance::In => -dir,
                    Variance::Invariant => 0,
                };
                match a {
                    Arg::Ty(at) => position(p, owner, at, sub, pos, bad),
                    Arg::Wild(e, s) => {
                        if let Some(e) = e {
                            position(p, owner, e, dir, pos, bad);
                        }
                        if let Some(s) = s {
                            position(p, owner, s, -dir, pos, bad);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
