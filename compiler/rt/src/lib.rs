//! Cleat's moving collector. Allocation goes through MMTk Immix, which relocates
//! unpinned objects. Stack slots are reported at each safepoint; the compiler also
//! emits LLVM `gc.statepoint` maps for those same slots.

const HEADER: usize = 16;
const ARRAY_BIT: u32 = 0x8000_0000;
const BOX_ID: u32 = 0x4000_0000;

pub struct CleatVM;

impl Default for CleatVM {
    fn default() -> Self {
        CleatVM
    }
}

pub struct CleatObjectModel;
pub struct CleatScanning;
pub struct CleatCollection;
pub struct CleatActivePlan;
pub struct CleatReferenceGlue;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CleatSlice;

impl VMBinding for CleatVM {
    type VMObjectModel = CleatObjectModel;
    type VMScanning = CleatScanning;
    type VMCollection = CleatCollection;
    type VMActivePlan = CleatActivePlan;
    type VMReferenceGlue = CleatReferenceGlue;
    type VMSlot = SimpleSlot;
    type VMMemorySlice = CleatSlice;
    const MIN_ALIGNMENT: usize = 8;
    const MAX_ALIGNMENT: usize = 8;
    const USE_ALLOCATION_OFFSET: bool = false;
}

impl ObjectModel<CleatVM> for CleatObjectModel {
    const GLOBAL_LOG_BIT_SPEC: VMGlobalLogBitSpec = VMGlobalLogBitSpec::side_first();
    const LOCAL_FORWARDING_POINTER_SPEC: VMLocalForwardingPointerSpec =
        VMLocalForwardingPointerSpec::side_first();
    const LOCAL_FORWARDING_BITS_SPEC: VMLocalForwardingBitsSpec =
        VMLocalForwardingBitsSpec::side_after(Self::LOCAL_FORWARDING_POINTER_SPEC.as_spec());
    const LOCAL_MARK_BIT_SPEC: VMLocalMarkBitSpec =
        VMLocalMarkBitSpec::side_after(Self::LOCAL_FORWARDING_BITS_SPEC.as_spec());
    const LOCAL_PINNING_BIT_SPEC: VMLocalPinningBitSpec =
        VMLocalPinningBitSpec::side_after(Self::LOCAL_MARK_BIT_SPEC.as_spec());
    const LOCAL_LOS_MARK_NURSERY_SPEC: VMLocalLOSMarkNurserySpec =
        VMLocalLOSMarkNurserySpec::side_after(Self::LOCAL_PINNING_BIT_SPEC.as_spec());
    fn copy(
        from: ObjectReference,
        semantics: CopySemantics,
        copy_context: &mut GCWorkerCopyContext<CleatVM>,
    ) -> ObjectReference {
        let bytes = Self::get_current_size(from);
        let addr = copy_context.alloc_copy(from, bytes, 8, 0, semantics);
        let to = unsafe { ObjectReference::from_raw_address_unchecked(addr) };
        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                from.to_raw_address().to_ptr(),
                addr.to_mut_ptr(),
                bytes,
            );
        }
        MOVED.fetch_add(1, Ordering::Relaxed);
        to
    }
    fn copy_to(from: ObjectReference, to: ObjectReference, _region: Address) -> Address {
        let bytes = Self::get_current_size(from);
        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                from.to_raw_address().to_ptr(),
                to.to_raw_address().to_mut_ptr(),
                bytes,
            );
        }
        MOVED.fetch_add(1, Ordering::Relaxed);
        to.to_raw_address() + bytes
    }

    fn get_reference_when_copied_to(_from: ObjectReference, to: Address) -> ObjectReference {
        unsafe { ObjectReference::from_raw_address_unchecked(to) }
    }

    fn get_current_size(object: ObjectReference) -> usize {
        header(object).size as usize
    }

    fn get_size_when_copied(object: ObjectReference) -> usize {
        Self::get_current_size(object)
    }

    fn get_align_when_copied(_object: ObjectReference) -> usize {
        8
    }

    fn get_align_offset_when_copied(_object: ObjectReference) -> usize {
        0
    }

    fn get_type_descriptor(_reference: ObjectReference) -> &'static [i8] {
        &[]
    }

    const UNIFIED_OBJECT_REFERENCE_ADDRESS: bool = true;

    const OBJECT_REF_OFFSET_LOWER_BOUND: isize = 0;

    fn ref_to_object_start(object: ObjectReference) -> Address {
        object.to_raw_address()
    }

    fn ref_to_header(object: ObjectReference) -> Address {
        object.to_raw_address()
    }

    fn dump_object(_object: ObjectReference) {}
}

impl Scanning<CleatVM> for CleatScanning {
    fn scan_object<SV: SlotVisitor<SimpleSlot>>(
        _tls: VMWorkerThread,
        object: ObjectReference,
        slot_visitor: &mut SV,
    ) {
        let header = header(object);
        let payload = object.to_raw_address() + HEADER;
        if header.class_id & ARRAY_BIT != 0 {
            let len = unsafe { (payload.to_ptr::<u32>()).read() } as usize;
            let slots = payload + 8usize;
            for i in 0..len {
                slot_visitor.visit_slot(SimpleSlot::from_address(slots + i * BYTES_IN_ADDRESS));
            }
            return;
        }
        for i in 0..header.nrefs as usize {
            slot_visitor.visit_slot(SimpleSlot::from_address(payload + i * BYTES_IN_ADDRESS));
        }
    }

    fn notify_initial_thread_scan_complete(_partial_scan: bool, _tls: VMWorkerThread) {}

    fn scan_roots_in_mutator_thread(
        _tls: VMWorkerThread,
        _mutator: &'static mut Mutator<CleatVM>,
        mut factory: impl RootsWorkFactory<SimpleSlot>,
    ) {
        let slots = ROOTS.lock().unwrap().clone();
        if !slots.is_empty() {
            factory.create_process_roots_work(slots);
        }
    }

    fn scan_vm_specific_roots(_tls: VMWorkerThread, _factory: impl RootsWorkFactory<SimpleSlot>) {}

    fn supports_return_barrier() -> bool {
        false
    }

    fn prepare_for_roots_re_scanning() {}
}

impl Collection<CleatVM> for CleatCollection {
    fn stop_all_mutators<F>(_tls: VMWorkerThread, mut mutator_visitor: F)
    where
        F: FnMut(&'static mut Mutator<CleatVM>),
    {
        mutator_visitor(mutator());
    }

    fn resume_mutators(_tls: VMWorkerThread) {
        EPOCH.fetch_add(1, Ordering::Release);
        COND.notify_all();
    }

    fn block_for_gc(_tls: VMMutatorThread) {
        let start = EPOCH.load(Ordering::Acquire);
        let mut guard = LOCK.lock().unwrap();
        while EPOCH.load(Ordering::Acquire) == start {
            guard = COND.wait(guard).unwrap();
        }
    }

    fn spawn_gc_thread(_tls: VMThread, ctx: GCThreadContext<CleatVM>) {
        match ctx {
            GCThreadContext::Worker(worker) => {
                let tls = VMWorkerThread(VMThread(
                    mmtk::util::opaque_pointer::OpaquePointer::from_address(Address::from_mut_ptr(
                        2usize as *mut u8,
                    )),
                ));
                let mmtk = mmtk_instance();
                std::thread::Builder::new()
                    .name("cleat-gc".into())
                    .spawn(move || worker.run(tls, mmtk))
                    .expect("gc worker");
            }
        }
    }

    fn out_of_memory(_tls: VMThread, _err_kind: AllocationError) {
        OOM.store(true, Ordering::Relaxed);
    }
}

impl ActivePlan<CleatVM> for CleatActivePlan {
    fn is_mutator(_tls: VMThread) -> bool {
        true
    }

    fn mutator(_tls: VMMutatorThread) -> &'static mut Mutator<CleatVM> {
        mutator()
    }

    fn mutators<'a>() -> Box<dyn Iterator<Item = &'a mut Mutator<CleatVM>> + 'a> {
        Box::new(std::iter::once(mutator()))
    }

    fn number_of_mutators() -> usize {
        1
    }
}

impl ReferenceGlue<CleatVM> for CleatReferenceGlue {
    type FinalizableType = ObjectReference;

    fn clear_referent(_new_reference: ObjectReference) {}

    fn get_referent(_object: ObjectReference) -> Option<ObjectReference> {
        None
    }

    fn set_referent(_reff: ObjectReference, _referent: ObjectReference) {}

    fn enqueue_references(_references: &[ObjectReference], _tls: VMWorkerThread) {}
}

impl MemorySlice for CleatSlice {
    type SlotType = SimpleSlot;
    type SlotIterator = std::iter::Empty<SimpleSlot>;

    fn iter_slots(&self) -> Self::SlotIterator {
        std::iter::empty()
    }

    fn object(&self) -> Option<ObjectReference> {
        None
    }

    fn start(&self) -> Address {
        Address::ZERO
    }

    fn bytes(&self) -> usize {
        0
    }

    fn copy(_src: &Self, _tgt: &Self) {}
}

#[repr(C)]
struct ObjHeader {
    size: u32,
    class_id: u32,
    nrefs: u32,
    flags: u32,
}

fn header(object: ObjectReference) -> &'static ObjHeader {
    unsafe { &*object.to_raw_address().to_ptr::<ObjHeader>() }
}

fn header_mut(addr: Address) -> &'static mut ObjHeader {
    unsafe { &mut *addr.to_mut_ptr::<ObjHeader>() }
}

static MOVED: AtomicUsize = AtomicUsize::new(0);
static OOM: AtomicBool = AtomicBool::new(false);
static NEXT_ID: AtomicU32 = AtomicU32::new(1);
static EPOCH: AtomicUsize = AtomicUsize::new(0);
static LOCK: Mutex<()> = Mutex::new(());
static COND: Condvar = Condvar::new();
static ROOTS: Mutex<Vec<SimpleSlot>> = Mutex::new(Vec::new());
static NOTES: Mutex<Vec<(u32, usize)>> = Mutex::new(Vec::new());
static mut MUTATOR: *mut Mutator<CleatVM> = std::ptr::null_mut();
static mut MM: *const MMTK<CleatVM> = std::ptr::null();

fn mutator() -> &'static mut Mutator<CleatVM> {
    unsafe { &mut *MUTATOR }
}

fn ensure() -> &'static MMTK<CleatVM> {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let mut builder = MMTKBuilder::new_no_env_vars();
        for (key, value) in [
            ("plan", "Immix"),
            ("threads", "1"),
            ("immix_always_defrag", "true"),
            ("immix_defrag_every_block", "true"),
            ("gc_trigger", "DynamicHeapSize:256k,256m"),
            ("stress_factor", "4096"),
        ] {
            if !builder.set_option(key, value) {
                eprintln!("cleat: MMTk rejected {key}={value}");
                std::process::exit(1);
            }
        }
        let mmtk: &'static MMTK<CleatVM> = Box::leak(Box::new(builder.build()));
        unsafe {
            MM = mmtk;
        }
        let tls = VMThread(mmtk::util::opaque_pointer::OpaquePointer::from_address(
            Address::from_mut_ptr(1usize as *mut u8),
        ));
        memory_manager::initialize_collection(mmtk, tls);
        let boxed = memory_manager::bind_mutator(mmtk, VMMutatorThread(tls));
        unsafe {
            MUTATOR = Box::into_raw(boxed);
        }
    });
    unsafe { &*MM }
}

fn remember_roots(roots: *mut *mut u8, nroots: u32) {
    let mut slots = ROOTS.lock().unwrap();
    slots.clear();
    if roots.is_null() || nroots == 0 {
        return;
    }
    for i in 0..nroots as isize {
        let slot = unsafe { roots.offset(i).read() } as *mut Address;
        if !slot.is_null() {
            slots.push(SimpleSlot::from_address(Address::from_mut_ptr(slot)));
        }
    }
}

fn allocate(total: usize, class_id: u32, nrefs: u32, roots: *mut *mut u8, nroots: u32) -> *mut u8 {
    let mmtk = ensure();
    remember_roots(roots, nroots);
    let bytes = (total + 7) & !7;
    let addr = alloc::<CleatVM>(mutator(), bytes, 8, 0, AllocationSemantics::Default);
    if addr.is_zero() || OOM.load(Ordering::Relaxed) {
        std::process::exit(1);
    }
    unsafe {
        std::ptr::write_bytes(addr.to_mut_ptr::<u8>(), 0, bytes);
    }
    let object = unsafe { ObjectReference::from_raw_address_unchecked(addr) };
    post_alloc::<CleatVM>(mutator(), object, bytes, AllocationSemantics::Default);
    let header = header_mut(addr);
    header.size = bytes as u32;
    header.class_id = class_id;
    header.nrefs = nrefs;
    header.flags = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let _ = mmtk;
    addr.to_mut_ptr()
}

#[no_mangle]
pub extern "C" fn cleat_alloc(
    payload: u32,
    class_id: u32,
    nrefs: u32,
    roots: *mut *mut u8,
    nroots: u32,
) -> *mut u8 {
    allocate(HEADER + payload as usize, class_id, nrefs, roots, nroots)
}

#[no_mangle]
pub extern "C" fn cleat_alloc_array(
    len: i32,
    elem_class: i32,
    roots: *mut *mut u8,
    nroots: u32,
) -> *mut u8 {
    if len < 0 {
        std::process::exit(1);
    }
    let bytes = 8 + (len as usize) * BYTES_IN_ADDRESS;
    let id = ARRAY_BIT | (elem_class as u32 & 0xffff);
    let obj = allocate(HEADER + bytes, id, 0, roots, nroots);
    unsafe {
        let head = obj.add(HEADER) as *mut u32;
        head.write(len as u32);
        head.add(1).write(elem_class as u32);
    }
    obj
}

#[no_mangle]
pub extern "C" fn cleat_box_i32(value: i32, roots: *mut *mut u8, nroots: u32) -> *mut u8 {
    let obj = allocate(HEADER + 4, BOX_ID, 0, roots, nroots);
    unsafe {
        (obj.add(HEADER) as *mut i32).write(value);
    }
    obj
}

#[no_mangle]
pub extern "C" fn cleat_pin(object: *mut u8) {
    let _ = ensure();
    if object.is_null() {
        return;
    }
    if let Some(object) = ObjectReference::from_raw_address(Address::from_mut_ptr(object)) {
        memory_manager::pin_object(object);
    }
}

#[no_mangle]
pub extern "C" fn cleat_note(object: *mut u8) {
    if object.is_null() {
        return;
    }
    let id = unsafe { (object as *const ObjHeader).read().flags };
    NOTES.lock().unwrap().push((id, object as usize));
}

#[no_mangle]
pub extern "C" fn cleat_moved(object: *mut u8) -> i32 {
    if object.is_null() {
        return 0;
    }
    let id = unsafe { (object as *const ObjHeader).read().flags };
    let notes = NOTES.lock().unwrap();
    if let Some((_, original)) = notes.iter().rev().find(|(noted, _)| *noted == id) {
        if *original != object as usize {
            1
        } else {
            0
        }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn cleat_move_count() -> i32 {
    MOVED.load(Ordering::Relaxed) as i32
}

use mmtk::memory_manager::{alloc, post_alloc};
use mmtk::util::alloc::AllocationError;
use mmtk::util::constants::BYTES_IN_ADDRESS;
use mmtk::util::copy::{CopySemantics, GCWorkerCopyContext};
use mmtk::util::{Address, ObjectReference, VMMutatorThread, VMThread, VMWorkerThread};
use mmtk::vm::slot::{MemorySlice, SimpleSlot};
use mmtk::vm::{
    ActivePlan, Collection, GCThreadContext, ObjectModel, ReferenceGlue, RootsWorkFactory,
    Scanning, SlotVisitor, VMBinding, VMGlobalLogBitSpec, VMLocalForwardingBitsSpec,
    VMLocalForwardingPointerSpec, VMLocalLOSMarkNurserySpec, VMLocalMarkBitSpec,
    VMLocalPinningBitSpec,
};
pub use mmtk::MMTKBuilder;
use mmtk::{memory_manager, AllocationSemantics, Mutator, MMTK};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, Once};

fn mmtk_instance() -> &'static MMTK<CleatVM> {
    unsafe { &*MM }
}
