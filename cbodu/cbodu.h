#pragma once

#include <stdint.h>

typedef struct cbodu_state cbodu_state;
typedef void (*cbodu_fn)(cbodu_state*);

extern uint64_t cbodu_number(cbodu_state*, int64_t);
extern uint64_t cbodu_float(cbodu_state*, double);
extern uint64_t cbodu_string(cbodu_state*, const char*);
extern uint64_t cbodu_boolean(cbodu_state*, uint8_t);
extern uint64_t cbodu_null(cbodu_state*);
extern uint8_t cbodu_iserr(cbodu_state*);
extern uint64_t cbodu_geterr(cbodu_state*);
extern void cbodu_seterr(cbodu_state*, uint64_t);
extern void cbodu_setret(cbodu_state*, uint64_t);
extern uint64_t cbodu_getinternal(cbodu_state*, uint64_t);
extern uint64_t cbodu_argslen(cbodu_state*);
extern uint64_t cbodu_getarg(cbodu_state*, uint64_t);
extern const char* cbodu_tostring(cbodu_state*, uint64_t);
extern int64_t cbodu_tonumber(cbodu_state*, uint64_t);
extern double cbodu_tofloat(cbodu_state*, uint64_t);
extern uint8_t cbodu_toboolean(cbodu_state*, uint64_t);
extern uint64_t cbodu_newobject(cbodu_state*);
extern uint64_t cbodu_get(cbodu_state*, uint64_t, uint64_t);
extern uint64_t cbodu_set(cbodu_state*, uint64_t, uint64_t, uint64_t);
extern uint64_t cbodu_obj_getmetaobj(cbodu_state*, uint64_t);
extern uint64_t cbodu_obj_setmetaobj(cbodu_state*, uint64_t, uint64_t);
extern uint64_t cbodu_obj_makegetset(cbodu_state*, uint64_t, uint64_t, uint64_t, uint64_t);
extern uint64_t cbodu_obj_getinternal(cbodu_state*, uint64_t, uint64_t);
extern uint64_t cbodu_obj_setinternal(cbodu_state*, uint64_t, uint64_t, uint64_t);
extern uint64_t cbodu_newfn(cbodu_state*, cbodu_fn);