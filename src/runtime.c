// 🐍 Snask Modular Runtime Entry Point
// Este arquivo unifica os módulos do novo runtime.

#include "runtime/rt_base.h"
#include "runtime/rt_gc.c"
#include "runtime/rt_obj.c"
#include "runtime/rt_io.c"
#include "runtime/rt_sfs.c"
#include "runtime/rt_json.c"
#include "runtime/rt_http.c"
#include "runtime/rt_gui.c"
#include "runtime/rt_sys.c"
#include "runtime/rt_auth.c"
#include "runtime/rt_blaze.c"

// Funções de compatibilidade ou stubs que ainda não foram modularizados
// podem ser adicionados aqui ou em rt_base.c
