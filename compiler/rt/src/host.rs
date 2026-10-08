//! What the host supplies: the console, files, the process, threads, locks and atomic
//! cells (chapter 13), and the data behind the mirrors of section 8.9.

#![allow(non_snake_case)]

use crate::abi::*;
use crate::gc::{self, Temp};
use crate::object::*;
use crate::types::{self, Td};
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicPtr, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

// ---- console, files, process ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Console_write_Int_String(ctx: *mut Ctx, stream: i64, s: Obj) {
    unsafe {
        let text = to_rust(s);
        gc::blocking(ctx, || {
            if stream == 2 {
                let _ = std::io::stdout().flush();
                let _ = std::io::stderr().write_all(text.as_bytes());
            } else {
                let mut out = std::io::stdout().lock();
                let _ = out.write_all(text.as_bytes());
                let _ = out.flush();
            }
        });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Console_readLine(ctx: *mut Ctx) -> Obj {
    unsafe {
        let line = gc::blocking(ctx, || {
            let mut line = String::new();
            match std::io::stdin().lock().read_line(&mut line) {
                Ok(n) if n > 0 => Some(line),
                _ => None,
            }
        });
        match line {
            Some(mut l) => {
                while l.ends_with('\n') || l.ends_with('\r') {
                    l.pop();
                }
                new_str(ctx, &l)
            }
            None => std::ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_File_readBytes_String(ctx: *mut Ctx, path: Obj) -> Obj {
    unsafe {
        let path = to_rust(path);
        let read = gc::blocking(ctx, || std::fs::read(&path));
        match read {
            Ok(bytes) => {
                let a = cl_array_new(ctx, array_type(types::wk_type(K_U8)), bytes.len() as i64);
                if !a.is_null() {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), at::<u8>(a, ELEMS), bytes.len());
                }
                a
            }
            Err(err) => {
                raise(ctx, X_IO, &format!("{path}: {err}"));
                std::ptr::null_mut()
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_File_writeBytes_String_UInt8Array(ctx: *mut Ctx, path: Obj, bytes: Obj) {
    unsafe {
        let path = to_rust(path);
        let data = byte_slice(bytes).to_vec();
        if let Err(err) = gc::blocking(ctx, || std::fs::write(&path, &data)) {
            raise(ctx, X_IO, &format!("{path}: {err}"));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Process_halt_Int(_ctx: *mut Ctx, status: i64) {
    let _ = std::io::stdout().flush();
    std::process::exit(status as i32);
}

// ---- threads ----

struct ThreadHost {
    started: AtomicBool,
    done: Mutex<bool>,
    wake: Condvar,
}

struct LockHost {
    /// The thread that holds the lock, or 0, and how many times it has taken it.
    state: Mutex<(u64, u64)>,
    free: Condvar,
}

/// Frees the host object that a `Lock`, a `Condition` or a `Thread` holds.
pub unsafe fn free_host(kind: u32, p: *mut u8) {
    unsafe {
        if p.is_null() {
            return;
        }
        match kind {
            K_LOCK => drop(Box::from_raw(p as *mut LockHost)),
            K_CONDITION => drop(Box::from_raw(p as *mut Condvar)),
            _ => drop(Arc::from_raw(p as *const ThreadHost)),
        }
    }
}

unsafe fn thread_host(t: Obj) -> &'static ThreadHost {
    unsafe { &**at::<*const ThreadHost>(t, BODY) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Thread_create(_ctx: *mut Ctx) -> *mut u8 {
    Arc::into_raw(Arc::new(ThreadHost { started: AtomicBool::new(false), done: Mutex::new(false), wake: Condvar::new() })) as *mut u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Thread_launch(ctx: *mut Ctx, t: Obj) {
    unsafe {
        let raw = *at::<*const ThreadHost>(t, BODY);
        Arc::increment_strong_count(raw);
        let host = Arc::from_raw(raw);
        if host.started.swap(true, Ordering::AcqRel) {
            raise(ctx, X_ILLEGAL_STATE, "the thread was already started");
            return;
        }
        // The object stays alive until the new thread holds it.
        gc::world().pinned.push(t);
        let object = t as usize;
        let spawned = std::thread::Builder::new().stack_size(gc::STACK).spawn(move || {
            let me = gc::new_ctx();
            let t = object as Obj;
            (*me).thread = t;
            {
                let mut w = gc::world();
                if let Some(i) = w.pinned.iter().position(|o| *o == t) {
                    w.pinned.swap_remove(i);
                }
            }
            (gc::program().thread_run)(me, t);
            (*me).exc = std::ptr::null_mut();
            *host.done.lock().unwrap_or_else(|e| e.into_inner()) = true;
            host.wake.notify_all();
            gc::drop_ctx(me);
        });
        if spawned.is_err() {
            raise(ctx, X_ILLEGAL_STATE, "the host could not start a thread");
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Thread_finished_Int(ctx: *mut Ctx, t: Obj, milliseconds: i64) -> u8 {
    unsafe {
        let host = thread_host(t);
        if !host.started.load(Ordering::Acquire) {
            return 0;
        }
        gc::blocking(ctx, || {
            let done = host.done.lock().unwrap_or_else(|e| e.into_inner());
            if *done {
                return 1;
            }
            let (done, _) = host.wake.wait_timeout(done, Duration::from_millis(milliseconds.max(0) as u64)).unwrap_or_else(|e| e.into_inner());
            *done as u8
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Thread_isAlive(_ctx: *mut Ctx, t: Obj) -> u8 {
    unsafe {
        let host = thread_host(t);
        (host.started.load(Ordering::Acquire) && !*host.done.lock().unwrap_or_else(|e| e.into_inner())) as u8
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Thread_running(ctx: *mut Ctx) -> Obj {
    unsafe { (*ctx).thread }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Thread_adopt_Thread(ctx: *mut Ctx, t: Obj) {
    unsafe {
        thread_host(t).started.store(true, Ordering::Release);
        (*ctx).thread = t;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Thread_pause_Int(ctx: *mut Ctx, milliseconds: i64) {
    unsafe { gc::blocking(ctx, || std::thread::sleep(Duration::from_millis(milliseconds.max(0) as u64))) }
}

// ---- locks ----

unsafe fn lock_host(l: Obj) -> &'static LockHost {
    unsafe { &**at::<*const LockHost>(l, BODY) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Lock_create(_ctx: *mut Ctx) -> *mut u8 {
    Box::into_raw(Box::new(LockHost { state: Mutex::new((0, 0)), free: Condvar::new() })) as *mut u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Lock_lock(ctx: *mut Ctx, l: Obj) {
    unsafe {
        let host = lock_host(l);
        let me = (*ctx).id;
        {
            let mut s = host.state.lock().unwrap_or_else(|e| e.into_inner());
            if s.0 == 0 || s.0 == me {
                *s = (me, s.1 + 1);
                return;
            }
        }
        gc::blocking(ctx, || {
            let mut s = host.state.lock().unwrap_or_else(|e| e.into_inner());
            while s.0 != 0 {
                s = host.free.wait(s).unwrap_or_else(|e| e.into_inner());
            }
            *s = (me, 1);
        });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Lock_unlock(ctx: *mut Ctx, l: Obj) {
    unsafe {
        let host = lock_host(l);
        let mut s = host.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.0 != (*ctx).id {
            drop(s);
            raise(ctx, X_ILLEGAL_STATE, "the running thread does not hold the lock");
            return;
        }
        s.1 -= 1;
        if s.1 == 0 {
            s.0 = 0;
            host.free.notify_all();
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Lock_held(ctx: *mut Ctx, l: Obj) -> u8 {
    unsafe { (lock_host(l).state.lock().unwrap_or_else(|e| e.into_inner()).0 == (*ctx).id) as u8 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Lock_park_Pointer_Int(ctx: *mut Ctx, l: Obj, condition: *mut u8, milliseconds: i64) {
    unsafe {
        let host = lock_host(l);
        let cond = &*(condition as *const Condvar);
        let me = (*ctx).id;
        gc::blocking(ctx, || {
            let mut s = host.state.lock().unwrap_or_else(|e| e.into_inner());
            let held = s.1;
            *s = (0, 0);
            host.free.notify_all();
            let (mut s, _) = cond.wait_timeout(s, Duration::from_millis(milliseconds.max(0) as u64)).unwrap_or_else(|e| e.into_inner());
            while s.0 != 0 {
                s = host.free.wait(s).unwrap_or_else(|e| e.into_inner());
            }
            *s = (me, held);
        });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Condition_create(_ctx: *mut Ctx) -> *mut u8 {
    Box::into_raw(Box::new(Condvar::new())) as *mut u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Condition_wake_Boolean(_ctx: *mut Ctx, c: Obj, all: u8) {
    unsafe {
        let cond = &**at::<*const Condvar>(c, BODY);
        if all != 0 {
            cond.notify_all();
        } else {
            cond.notify_one();
        }
    }
}

// ---- atomic cells ----

unsafe fn cell(o: Obj) -> &'static AtomicI64 {
    unsafe { &*at::<AtomicI64>(o, BODY) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_AtomicInt_get(_ctx: *mut Ctx, o: Obj) -> i64 {
    unsafe { cell(o).load(Ordering::SeqCst) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_AtomicInt_set_Int(_ctx: *mut Ctx, o: Obj, v: i64) {
    unsafe { cell(o).store(v, Ordering::SeqCst) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_AtomicInt_compareAndSet_Int_Int(_ctx: *mut Ctx, o: Obj, expected: i64, update: i64) -> u8 {
    unsafe { cell(o).compare_exchange(expected, update, Ordering::SeqCst, Ordering::SeqCst).is_ok() as u8 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_AtomicInt_addAndGet_Int(ctx: *mut Ctx, o: Obj, delta: i64) -> i64 {
    unsafe {
        let c = cell(o);
        loop {
            let now = c.load(Ordering::SeqCst);
            let Some(next) = now.checked_add(delta) else {
                raise(ctx, X_ARITHMETIC, &format!("{now} + {delta} is not a value of Int"));
                return 0;
            };
            if c.compare_exchange(now, next, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return next;
            }
        }
    }
}

unsafe fn ref_cell(o: Obj) -> &'static AtomicPtr<Header> {
    unsafe { &*at::<AtomicPtr<Header>>(o, BODY) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Atomic_get(_ctx: *mut Ctx, o: Obj) -> Obj {
    unsafe { ref_cell(o).load(Ordering::SeqCst) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Atomic_set_Object(_ctx: *mut Ctx, o: Obj, v: Obj) {
    unsafe { ref_cell(o).store(v, Ordering::SeqCst) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Atomic_compareAndSet_Object_Object(_ctx: *mut Ctx, o: Obj, expected: Obj, update: Obj) -> u8 {
    unsafe {
        let c = ref_cell(o);
        loop {
            let now = c.load(Ordering::SeqCst);
            if !same_identity(now, expected) {
                return 0;
            }
            if c.compare_exchange(now, update, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return 1;
            }
        }
    }
}

// ---- class objects and mirrors (section 8.9) ----

unsafe fn class_td(c: Obj) -> &'static types::TypeDesc {
    unsafe { &**at::<Td>(c, BODY) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_getName(ctx: *mut Ctx, c: Obj) -> Obj {
    unsafe { new_str(ctx, class_td(c).name()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_getSuperclass(_ctx: *mut Ctx, c: Obj) -> Obj {
    unsafe {
        let td = class_td(c);
        let info = td.class();
        if !info.is(F_SUPERCLASS) || info.is(F_INTERFACE) {
            return std::ptr::null_mut();
        }
        class_object(types::eval(info.supers()[0], td.args.as_ptr()))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_getInterfaces(ctx: *mut Ctx, c: Obj) -> Obj {
    unsafe {
        let td = class_td(c);
        let info = td.class();
        let skip = if info.is(F_SUPERCLASS) { 1 } else { 0 };
        let list: Vec<Obj> = info.supers()[skip..].iter().map(|s| class_object(types::eval(*s, td.args.as_ptr()))).collect();
        let a = cl_array_new(ctx, array_type(types::wk_type(K_CLASS)), list.len() as i64);
        if !a.is_null() {
            for (i, o) in list.iter().enumerate() {
                *at::<Obj>(a, ELEMS).add(i) = *o;
            }
        }
        a
    }
}

unsafe fn find_ann(ctx: *mut Ctx, anns: *const AnnInfo, n: u32, wanted: *const ClassInfo) -> Option<Obj> {
    unsafe { slice(anns, n).iter().find(|a| a.class == wanted).map(|a| (a.make)(ctx)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_getAnnotation(ctx: *mut Ctx, targs: *const Td, c: Obj) -> Obj {
    unsafe {
        let wanted = (**targs).class;
        let td = class_td(c);
        if let Some(o) = find_ann(ctx, td.class().anns, td.class().nanns, wanted) {
            return o;
        }
        if (*wanted).is(F_INHERITED) {
            // A tag declared `@Inherited` is carried by what extends a marked type.
            for up in td.supers() {
                let info = (**up).class();
                if let Some(o) = find_ann(ctx, info.anns, info.nanns, wanted) {
                    return o;
                }
            }
        }
        std::ptr::null_mut()
    }
}

unsafe fn int_array(ctx: *mut Ctx, values: &[i64]) -> Obj {
    unsafe {
        let a = cl_array_new(ctx, array_type(types::wk_type(K_I64)), values.len() as i64);
        if !a.is_null() {
            std::ptr::copy_nonoverlapping(values.as_ptr(), at::<i64>(a, ELEMS), values.len());
        }
        a
    }
}

unsafe fn carrying(members: &'static [MemberInfo], wanted: *const ClassInfo) -> Vec<i64> {
    unsafe { members.iter().enumerate().filter(|(_, m)| slice(m.anns, m.nanns).iter().any(|a| a.class == wanted)).map(|(i, _)| i as i64).collect() }
}

unsafe fn fields(c: Obj) -> &'static [MemberInfo] {
    unsafe { slice(class_td(c).class().mfields, class_td(c).class().nmfields) }
}

unsafe fn methods(c: Obj) -> &'static [MemberInfo] {
    unsafe { slice(class_td(c).class().mmethods, class_td(c).class().nmmethods) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_annotatedFields(ctx: *mut Ctx, targs: *const Td, c: Obj) -> Obj {
    unsafe { int_array(ctx, &carrying(fields(c), (**targs).class)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_annotatedMethods(ctx: *mut Ctx, targs: *const Td, c: Obj) -> Obj {
    unsafe { int_array(ctx, &carrying(methods(c), (**targs).class)) }
}

unsafe fn member_name(ctx: *mut Ctx, m: &MemberInfo) -> Obj {
    unsafe { new_str(ctx, std::str::from_utf8_unchecked(std::slice::from_raw_parts(m.name, m.name_len as usize))) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_fieldName_Int(ctx: *mut Ctx, c: Obj, i: i64) -> Obj {
    unsafe { member_name(ctx, &fields(c)[i as usize]) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_fieldIsStatic_Int(_ctx: *mut Ctx, c: Obj, i: i64) -> u8 {
    unsafe { (fields(c)[i as usize].flags & M_STATIC != 0) as u8 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_fieldAnnotation_Int(ctx: *mut Ctx, targs: *const Td, c: Obj, i: i64) -> Obj {
    unsafe {
        let m = &fields(c)[i as usize];
        find_ann(ctx, m.anns, m.nanns, (**targs).class).unwrap_or(std::ptr::null_mut())
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_fieldGet_Int_Object(ctx: *mut Ctx, c: Obj, i: i64, target: Obj) -> Obj {
    unsafe { (fields(c)[i as usize].get.expect("a field mirror"))(ctx, target) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_fieldSet_Int_Object_Object(ctx: *mut Ctx, c: Obj, i: i64, target: Obj, value: Obj) {
    unsafe { (fields(c)[i as usize].set.expect("a field mirror"))(ctx, target, value) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_methodName_Int(ctx: *mut Ctx, c: Obj, i: i64) -> Obj {
    unsafe { member_name(ctx, &methods(c)[i as usize]) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_methodIsStatic_Int(_ctx: *mut Ctx, c: Obj, i: i64) -> u8 {
    unsafe { (methods(c)[i as usize].flags & M_STATIC != 0) as u8 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_parameterCount_Int(_ctx: *mut Ctx, c: Obj, i: i64) -> i64 {
    unsafe { methods(c)[i as usize].nparams as i64 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_parameterName_Int_Int(ctx: *mut Ctx, c: Obj, i: i64, p: i64) -> Obj {
    unsafe {
        let m = &methods(c)[i as usize];
        let prm = &slice(m.params, m.nparams)[p as usize];
        new_str(ctx, std::str::from_utf8_unchecked(std::slice::from_raw_parts(prm.name, prm.name_len as usize)))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_methodAnnotation_Int(ctx: *mut Ctx, targs: *const Td, c: Obj, i: i64) -> Obj {
    unsafe {
        let m = &methods(c)[i as usize];
        find_ann(ctx, m.anns, m.nanns, (**targs).class).unwrap_or(std::ptr::null_mut())
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_parameterAnnotation_Int_Int(ctx: *mut Ctx, targs: *const Td, c: Obj, i: i64, p: i64) -> Obj {
    unsafe {
        let m = &methods(c)[i as usize];
        let prm = &slice(m.params, m.nparams)[p as usize];
        find_ann(ctx, prm.anns, prm.nanns, (**targs).class).unwrap_or(std::ptr::null_mut())
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Class_methodInvoke_Int_Object_ObjectArray(ctx: *mut Ctx, c: Obj, i: i64, target: Obj, arguments: Obj) -> Obj {
    unsafe {
        let _a = Temp::new(ctx, arguments);
        (methods(c)[i as usize].invoke.expect("a method mirror"))(ctx, target, arguments)
    }
}

/// What a mirror's generated code calls to refuse a use (section 8.9).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_mirror_fail(ctx: *mut Ctx, kind: u32, text: *const u8, n: u64) {
    unsafe {
        let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(text, n as usize));
        raise(ctx, kind, s);
    }
}

/// Raises `ClassCastException` unless `o` is an instance of the class, whatever its
/// type arguments.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_cast_class(ctx: *mut Ctx, o: Obj, class: *const ClassInfo) -> Obj {
    unsafe {
        let ok = !o.is_null() && (*(*o).td).super_at(class).is_some();
        if !ok {
            let have = if o.is_null() { "null" } else { (*(*o).td).name() };
            raise(ctx, X_CLASS_CAST, &format!("a {have} is not a {}", (*class).name()));
        }
        o
    }
}

/// Before a foreign call: the thread counts as stopped while C code runs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_foreign_enter(ctx: *mut Ctx) {
    unsafe {
        let _w = gc::world();
        (*ctx).safe = true;
        gc::WAKE.notify_all();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_foreign_leave(ctx: *mut Ctx) {
    unsafe {
        let mut w = gc::world();
        while w.collecting {
            w = gc::WAKE.wait(w).unwrap_or_else(|e| e.into_inner());
        }
        (*ctx).safe = false;
    }
}
