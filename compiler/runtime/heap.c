#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/* Precise, non-moving collector. Reference fields are the first payload
   slots. An unboxed Int32 is not a pointer and is not scanned. */

enum { HEADER = 16, MAX_CLASS = 1024, MAX_ROOTS = 8192, ARRAY_BIT = 0x80000000, BOX_ID = 0x40000000 };

typedef struct Obj {
    uint32_t size;
    uint32_t class_id;
    uint32_t nrefs;
    uint32_t flags;
} Obj;

typedef struct Chunk {
    struct Chunk *next;
    size_t cap;
    size_t used;
    char data[];
} Chunk;

typedef struct ClassMeta {
    Obj header;
    uint32_t represented;
    uint32_t ann_count;
} ClassMeta;

static Chunk *chunks;
static Obj *freelist;
static void **roots[MAX_ROOTS];
static int nroots;
static int parents[MAX_CLASS];
static ClassMeta user_meta[MAX_CLASS];
static ClassMeta array_meta[MAX_CLASS];
static ClassMeta box_meta;
static ClassMeta object_meta;
static size_t heap_limit_step = 64 * 1024;

static int in_heap(const void *p) {
    const char *c = p;
    for (Chunk *chunk = chunks; chunk; chunk = chunk->next) {
        if (c >= chunk->data && c < chunk->data + chunk->used) {
            return 1;
        }
    }
    return 0;
}

static void mark(Obj *obj) {
    if (!obj || !in_heap(obj) || (obj->flags & 1)) {
        return;
    }
    obj->flags |= 1;
    char *payload = (char *)(obj + 1);
    if (obj->class_id & ARRAY_BIT) {
        uint32_t len = *(uint32_t *)payload;
        void **slots = (void **)(payload + 8);
        for (uint32_t i = 0; i < len; i++) {
            mark(slots[i]);
        }
        return;
    }
    void **slots = (void **)payload;
    for (uint32_t i = 0; i < obj->nrefs; i++) {
        mark(slots[i]);
    }
}

static void *payload_link(Obj *obj) {
    return obj + 1;
}

static void sweep(void) {
    freelist = NULL;
    for (Chunk *chunk = chunks; chunk; chunk = chunk->next) {
        size_t at = 0;
        while (at + HEADER <= chunk->used) {
            Obj *obj = (Obj *)(chunk->data + at);
            if (obj->size < HEADER || at + obj->size > chunk->used) {
                break;
            }
            if (obj->flags & 2) {
                at += obj->size;
                continue;
            }
            if (obj->flags & 1) {
                obj->flags &= ~1u;
            } else {
                obj->flags = 2;
                obj->nrefs = 0;
                *(Obj **)payload_link(obj) = freelist;
                freelist = obj;
            }
            at += obj->size;
        }
    }
}

static void collect(void) {
    for (int i = 0; i < nroots; i++) {
        if (roots[i]) {
            mark(*roots[i]);
        }
    }
    sweep();
}

static void *bump(size_t size) {
    for (Chunk *chunk = chunks; chunk; chunk = chunk->next) {
        size_t aligned = (chunk->used + 7u) & ~(size_t)7u;
        if (aligned + size <= chunk->cap) {
            chunk->used = aligned + size;
            return chunk->data + aligned;
        }
    }
    return NULL;
}

static int add_chunk(size_t need) {
    size_t cap = heap_limit_step;
    if (cap < need) {
        cap = (need + 4095u) & ~(size_t)4095u;
    }
    Chunk *chunk = malloc(sizeof(Chunk) + cap);
    if (!chunk) {
        return 0;
    }
    chunk->next = chunks;
    chunk->cap = cap;
    chunk->used = 0;
    chunks = chunk;
    if (heap_limit_step < (size_t)1 << 28) {
        heap_limit_step *= 2;
    }
    return 1;
}

static void *raw_alloc(size_t size, size_t *capacity) {
    size = (size + 7u) & ~(size_t)7u;
    for (int attempt = 0; attempt < 4; attempt++) {
        Obj **prev = &freelist;
        for (Obj *block = freelist; block; block = *(Obj **)payload_link(block)) {
            if ((block->flags & 2) && block->size >= size) {
                *prev = *(Obj **)payload_link(block);
                *capacity = block->size;
                return block;
            }
            prev = (Obj **)payload_link(block);
        }
        void *fresh = bump(size);
        if (fresh) {
            *capacity = size;
            return fresh;
        }
        collect();
        if (!add_chunk(size)) {
            exit(1);
        }
    }
    exit(1);
}

static Obj *alloc_obj(size_t payload, uint32_t class_id, uint32_t nrefs) {
    size_t total = (HEADER + payload + 7u) & ~(size_t)7u;
    size_t capacity = 0;
    Obj *obj = raw_alloc(total, &capacity);
    uint32_t keep = capacity > total ? (uint32_t)capacity : (uint32_t)total;
    memset(obj, 0, total);
    obj->size = keep;
    obj->class_id = class_id;
    obj->nrefs = nrefs;
    obj->flags = 0;
    return obj;
}

void cleat_push_root(void **slot) {
    if (nroots >= MAX_ROOTS) {
        exit(1);
    }
    roots[nroots++] = slot;
}

void cleat_pop_root(void) {
    if (nroots > 0) {
        nroots--;
    }
}

void cleat_set_parent(int id, int parent) {
    if (id > 0 && id < MAX_CLASS) {
        parents[id] = parent;
        user_meta[id].header.size = HEADER;
        user_meta[id].header.class_id = 0;
        user_meta[id].represented = (uint32_t)id;
        user_meta[id].ann_count = 0;
    }
}

void *cleat_alloc(uint32_t payload, uint32_t class_id, uint32_t nrefs) {
    return alloc_obj(payload, class_id, nrefs);
}

void *cleat_alloc_array(int32_t len, int32_t elem_class) {
    if (len < 0) {
        exit(1);
    }
    size_t bytes = 8u + (size_t)len * sizeof(void *);
    if (len != 0 && bytes / sizeof(void *) != (size_t)len + 1) {
        exit(1);
    }
    uint32_t id = ARRAY_BIT | ((uint32_t)elem_class & 0xffffu);
    Obj *obj = alloc_obj(bytes, id, 0);
    uint32_t *head = (uint32_t *)(obj + 1);
    head[0] = (uint32_t)len;
    head[1] = (uint32_t)elem_class;
    return obj;
}

void *cleat_box_i32(int32_t value) {
    Obj *obj = alloc_obj(4, BOX_ID, 0);
    *(int32_t *)(obj + 1) = value;
    return obj;
}

static int represented(void *class_obj) {
    return (int)((ClassMeta *)class_obj)->represented;
}

void *cleat_get_class(void *obj) {
    if (!obj) {
        return &object_meta;
    }
    Obj *o = obj;
    if (o->class_id == BOX_ID) {
        box_meta.represented = BOX_ID;
        return &box_meta;
    }
    if (o->class_id & ARRAY_BIT) {
        unsigned elem = o->class_id & 0xffffu;
        if (elem >= MAX_CLASS) {
            elem = 0;
        }
        array_meta[elem].represented = o->class_id;
        return &array_meta[elem];
    }
    unsigned id = o->class_id;
    if (id >= MAX_CLASS) {
        id = 0;
    }
    if (id == 0) {
        object_meta.represented = 0;
        return &object_meta;
    }
    user_meta[id].represented = id;
    return &user_meta[id];
}

int cleat_is_instance(void *class_obj, void *obj) {
    if (!class_obj || !obj || !in_heap(obj)) {
        return 0;
    }
    int want = represented(class_obj);
    int id = (int)((Obj *)obj)->class_id;
    for (int guard = 0; guard < 64; guard++) {
        if (id == want) {
            return 1;
        }
        if (id == BOX_ID || (id & ARRAY_BIT)) {
            id = 0;
            continue;
        }
        if (id <= 0 || id >= MAX_CLASS) {
            return 0;
        }
        id = parents[id];
        if (id == 0 && want == 0) {
            return 1;
        }
    }
    return 0;
}

int cleat_has(void *class_obj, void *annotation) {
    (void)annotation;
    if (!class_obj) {
        return 0;
    }
    return ((ClassMeta *)class_obj)->ann_count != 0;
}

void *cleat_array_load(void *array, int32_t index) {
    if (!array) {
        exit(1);
    }
    uint32_t *head = (uint32_t *)((Obj *)array + 1);
    if (index < 0 || (uint32_t)index >= head[0]) {
        exit(1);
    }
    void **slots = (void **)((char *)head + 8);
    return slots[index];
}

void cleat_array_store(void *array, int32_t index, void *value) {
    if (!array) {
        exit(1);
    }
    uint32_t *head = (uint32_t *)((Obj *)array + 1);
    if (index < 0 || (uint32_t)index >= head[0]) {
        exit(1);
    }
    void **slots = (void **)((char *)head + 8);
    slots[index] = value;
}
