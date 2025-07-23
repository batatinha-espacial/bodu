#pragma once

#include <stdint.h>

extern uint64_t cbodu_number(void*, int64_t);
extern uint64_t cbodu_float(void*, double);
extern uint64_t cbodu_string(void*, const char*);
extern uint64_t cbodu_boolean(void*, uint8_t);
extern uint64_t cbodu_null(void*);
extern uint8_t cbodu_iserr(void*);
extern uint64_t cbodu_geterr(void*);
extern void cbodu_seterr(void*, uint64_t);
extern void cbodu_setret(void*, uint64_t);
extern uint64_t cbodu_getinternal(void*, uint64_t);
extern uint64_t cbodu_argslen(void*);
extern uint64_t cbodu_getarg(void*, uint64_t);
extern const char* cbodu_tostring(void*, uint64_t);
extern int64_t cbodu_tonumber(void*, uint64_t);
extern double cbodu_tofloat(void*, uint64_t);
extern uint8_t cbodu_toboolean(void*, uint64_t);
extern uint64_t cbodu_newobject(void*);
extern uint64_t cbodu_get(void*, uint64_t, uint64_t);
extern uint64_t cbodu_set(void*, uint64_t, uint64_t, uint64_t);
extern uint64_t cbodu_obj_getmetaobj(void*, uint64_t);
extern uint64_t cbodu_obj_setmetaobj(void*, uint64_t, uint64_t);
extern uint64_t cbodu_obj_makegetset(void*, uint64_t, uint64_t, uint64_t, uint64_t);
