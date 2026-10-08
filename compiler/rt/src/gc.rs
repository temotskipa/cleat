//! Storage: allocation and a precise, non-moving mark and sweep collector.
//!
//! The roots are exact. Compiled code keeps every pointer it holds in the slots of a
//! frame linked from its thread's context, and the compiler lists the static fields
//! that hold pointers. A thread stops for a collection only where its frames are
//! complete: at an allocation, at a poll on a loop's back edge, and while it is blocked
//! in the runtime.

use crate::abi::*;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};

pub struct World {
    pub threads: Vec<*mut Ctx>,
    pub collecting: bool,
    /// The objects of threads that have ended.
    pub orphans: Vec<Obj>,
    /// Bytes allocated since the last collection, as far as threads have reported.
    pub since: usize,
    pub threshold: usize,
    /// Pointers the runtime itself holds: the exception a failed class keeps, for one.
    pub pinned: Vec<Obj>,
    pub collections: u64,
}

unsafe impl Send for World {}

pub static WORLD: Mutex<World> = Mutex::new(World {
    threads: Vec::new(),
    collecting: false,
    orphans: Vec::new(),
    since: 0,
    threshold: 16 << 20,
    pinned: Vec::new(),
    collections: 0,
});
pub static WAKE: Condvar = Condvar::new();

pub static mut PROGRAM: *const Program = std::ptr::null();
/// When nonzero, collect at every that-many-th allocation. Set by `CLEAT_GC_STRESS`.
pub static STRESS: AtomicUsize = AtomicUsize::new(0);
static STRESS_COUNT: AtomicUsize = AtomicUsize::new(0);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const REPORT: usize = 64 << 10;

pub fn world() -> MutexGuard<'static, World> {
    WORLD.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn program() -> &'static Program {
    unsafe { &*PROGRAM }
}

pub fn new_ctx() -> *mut Ctx {
    let ctx = Box::into_raw(Box::new(Ctx {
        exc: std::ptr::null_mut(),
        top: std::ptr::null_mut(),
        poll: std::sync::atomic::AtomicU32::new(0),
        _pad: 0,
        thread: std::ptr::null_mut(),
        temps: Vec::new(),
        allocated: Vec::new(),
        bytes: 0,
        safe: false,
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
    }));
    let mut w = world();
    while w.collecting {
        w = WAKE.wait(w).unwrap_or_else(|e| e.into_inner());
    }
    w.threads.push(ctx);
    ctx
}

pub unsafe fn drop_ctx(ctx: *mut Ctx) {
    unsafe {
        let mut w = world();
        while w.collecting {
            (*ctx).safe = true;
            WAKE.notify_all();
            w = WAKE.wait(w).unwrap_or_else(|e| e.into_inner());
        }
        let mut objects = std::mem::take(&mut (*ctx).allocated);
        w.orphans.append(&mut objects);
        w.since += (*ctx).bytes;
        w.threads.retain(|t| *t != ctx);
        WAKE.notify_all();
        drop(w);
        drop(Box::from_raw(ctx));
    }
}

/// Stops the thread while a collection runs.
pub unsafe fn park(ctx: *mut Ctx) {
    unsafe {
        let mut w = world();
        (*ctx).safe = true;
        WAKE.notify_all();
        while w.collecting {
            w = WAKE.wait(w).unwrap_or_else(|e| e.into_inner());
        }
        (*ctx).safe = false;
    }
}

/// Runs `f`, which may block, with the thread counted as stopped. `f` must not touch
/// an object.
pub unsafe fn blocking<R>(ctx: *mut Ctx, f: impl FnOnce() -> R) -> R {
    unsafe {
        {
            let _w = world();
            (*ctx).safe = true;
            WAKE.notify_all();
        }
        let r = f();
        let mut w = world();
        while w.collecting {
            w = WAKE.wait(w).unwrap_or_else(|e| e.into_inner());
        }
        (*ctx).safe = false;
        r
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_poll(ctx: *mut Ctx) {
    unsafe {
        if (*ctx).poll.load(Ordering::Relaxed) != 0 {
            park(ctx);
        }
    }
}

/// The storage an object occupies.
pub unsafe fn object_size(o: Obj) -> usize {
    unsafe {
        let td = &*(*o).td;
        let c = &*td.class;
        let len = || *((o as *const u8).add(BODY) as *const i64) as usize;
        match c.kind {
            K_ARRAY => ELEMS + len() * elem_size(td.elem_kind),
            K_STRING => ELEMS + len() * 4,
            k if is_machine(k) || k == K_RATIONAL || k == K_CLASS => 24,
            _ => (c.size as usize).max(BODY),
        }
    }
}

fn layout(size: usize) -> Option<Layout> {
    Layout::from_size_align(size.max(BODY), 16).ok()
}

/// Allocates zeroed storage for an object and sets its type. Returns null when no
/// storage can be found, even after a collection.
pub unsafe fn alloc(ctx: *mut Ctx, td: *const crate::types::TypeDesc, size: usize) -> Obj {
    unsafe {
        if (*ctx).poll.load(Ordering::Relaxed) != 0 {
            park(ctx);
        }
        let stress = STRESS.load(Ordering::Relaxed);
        if stress != 0 && STRESS_COUNT.fetch_add(1, Ordering::Relaxed) % stress == 0 {
            collect(ctx);
        }
        if (*ctx).bytes >= REPORT {
            let due = {
                let mut w = world();
                w.since += (*ctx).bytes;
                (*ctx).bytes = 0;
                w.since >= w.threshold
            };
            if due {
                collect(ctx);
            }
        }
        let Some(l) = layout(size) else { return std::ptr::null_mut() };
        let mut p = alloc_zeroed(l) as Obj;
        if p.is_null() {
            collect(ctx);
            p = alloc_zeroed(l) as Obj;
            if p.is_null() {
                return p;
            }
        }
        (*p).td = td;
        (*ctx).allocated.push(p);
        (*ctx).bytes += size;
        p
    }
}

/// An object outside the collected heap.
pub unsafe fn alloc_static(td: *const crate::types::TypeDesc, size: usize) -> Obj {
    unsafe {
        let p = alloc_zeroed(layout(size).expect("a static object")) as Obj;
        (*p).td = td;
        (*p).gc = GC_STATIC;
        p
    }
}

unsafe fn mark(stack: &mut Vec<Obj>, o: Obj) {
    unsafe {
        if o.is_null() || (*o).gc & (GC_MARK | GC_STATIC) != 0 {
            return;
        }
        (*o).gc |= GC_MARK;
        stack.push(o);
    }
}

unsafe fn trace(stack: &mut Vec<Obj>, o: Obj) {
    unsafe {
        let td = &*(*o).td;
        let c = &*td.class;
        let base = o as *const u8;
        match c.kind {
            K_ARRAY => {
                if td.elem_kind == 0 {
                    let len = *(base.add(BODY) as *const i64) as usize;
                    let elems = base.add(ELEMS) as *const Obj;
                    for i in 0..len {
                        mark(stack, *elems.add(i));
                    }
                }
            }
            K_STRING | K_RATIONAL | K_CLASS | K_UNIT => {}
            k if is_machine(k) => {}
            _ => {
                for off in slice(c.refs, c.nrefs) {
                    mark(stack, *(base.add(*off as usize) as *const Obj));
                }
            }
        }
    }
}

unsafe fn finalize(o: Obj) {
    unsafe {
        let c = &*(*(*o).td).class;
        let payload = *((o as *const u8).add(BODY) as *const *mut u8);
        match c.kind {
            K_RATIONAL => {
                if !payload.is_null() {
                    drop(Box::from_raw(payload as *mut num_rational::BigRational));
                }
            }
            K_LOCK | K_CONDITION | K_THREAD => crate::host::free_host(c.kind, payload),
            _ => {}
        }
    }
}

/// Stops every thread and collects. The calling thread's frames must be complete.
pub unsafe fn collect(ctx: *mut Ctx) {
    unsafe {
        let mut w = world();
        if w.collecting {
            // Another thread got there first: wait for it.
            (*ctx).safe = true;
            WAKE.notify_all();
            while w.collecting {
                w = WAKE.wait(w).unwrap_or_else(|e| e.into_inner());
            }
            (*ctx).safe = false;
            return;
        }
        w.collecting = true;
        for t in &w.threads {
            (**t).poll.store(1, Ordering::Relaxed);
        }
        (*ctx).safe = true;
        while w.threads.iter().any(|t| !(**t).safe) {
            w = WAKE.wait(w).unwrap_or_else(|e| e.into_inner());
        }

        let mut stack: Vec<Obj> = Vec::new();
        for t in &w.threads {
            let t = &**t;
            mark(&mut stack, t.exc);
            mark(&mut stack, t.thread);
            for o in &t.temps {
                mark(&mut stack, *o);
            }
            let mut f = t.top;
            while !f.is_null() {
                let slots = (f as *const u8).add(std::mem::size_of::<Frame>()) as *const Obj;
                for i in 0..(*f).n as usize {
                    mark(&mut stack, *slots.add(i));
                }
                f = (*f).prev;
            }
        }
        for o in &w.pinned {
            mark(&mut stack, *o);
        }
        if !PROGRAM.is_null() {
            let p = program();
            for root in slice(p.roots, p.nroots) {
                mark(&mut stack, **root);
            }
        }
        while let Some(o) = stack.pop() {
            trace(&mut stack, o);
        }

        let mut live = 0usize;
        let mut sweep = |list: &mut Vec<Obj>| {
            list.retain(|o| {
                let o = *o;
                let size = object_size(o);
                if (*o).gc & GC_MARK != 0 {
                    (*o).gc &= !GC_MARK;
                    live += size;
                    true
                } else {
                    finalize(o);
                    dealloc(o as *mut u8, layout(size).unwrap());
                    false
                }
            });
        };
        let threads = w.threads.clone();
        for t in &threads {
            sweep(&mut (**t).allocated);
            (**t).bytes = 0;
        }
        sweep(&mut w.orphans);

        w.since = 0;
        w.threshold = (live * 2).max(16 << 20);
        w.collections += 1;
        for t in &w.threads {
            (**t).poll.store(0, Ordering::Relaxed);
        }
        (*ctx).safe = false;
        w.collecting = false;
        WAKE.notify_all();
    }
}

/// Keeps an object alive until the guard is dropped, for runtime code that allocates
/// while it holds a pointer no frame holds.
pub struct Temp(*mut Ctx);

impl Temp {
    pub unsafe fn new(ctx: *mut Ctx, o: Obj) -> Temp {
        unsafe {
            (*ctx).temps.push(o);
        }
        Temp(ctx)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        unsafe {
            (*self.0).temps.pop();
        }
    }
}
