// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <dlfcn.h>
#include "muapi.h"
#include "mu-fastimpl.h"
#ifdef __APPLE__
    #define LIB_EXT ".dylib"
#elif __linux__
    #define LIB_EXT ".so"
#elif _WIN32
    #define LIB_EXT ".dll"
#endif
#define LIB_FILE_NAME(name) "lib" name LIB_EXT
int main(int argc, char** argv) {
    MuVM* mu_10;
    MuCtx* ctx_10;
    MuIRBuilder* bldr_10;
    MuID id_91;
    MuID id_92;
    MuID id_93;
    MuID id_94;
    MuID id_95;
    MuID id_96;
    MuID id_97;
    MuID id_98;
    MuID id_99;
    MuID id_100;
    mu_10 = mu_fastimpl_new_with_opts("init_mu --log-level=none --aot-emit-dir=emit");
    ctx_10 = mu_10->new_context(mu_10);
    bldr_10 = ctx_10->new_ir_builder(ctx_10);
    id_91 = bldr_10->gen_sym(bldr_10, "@i64");
    bldr_10->new_type_int(bldr_10, id_91, 0x00000040ull);
    id_92 = bldr_10->gen_sym(bldr_10, "@0x8d9f9c1d58324b55_i64");
    bldr_10->new_const_int(bldr_10, id_92, id_91, 0x8d9f9c1d58324b55ull);
    id_93 = bldr_10->gen_sym(bldr_10, "@0x0a_i64");
    bldr_10->new_const_int(bldr_10, id_93, id_91, 0x000000000000000aull);
    id_94 = bldr_10->gen_sym(bldr_10, "@sig__i64");
    bldr_10->new_funcsig(bldr_10, id_94, NULL, 0, (MuTypeNode [1]){id_91}, 1);
    id_95 = bldr_10->gen_sym(bldr_10, "@test_fnc");
    bldr_10->new_func(bldr_10, id_95, id_94);
    id_96 = bldr_10->gen_sym(bldr_10, "@test_fnc_v1");
    id_97 = bldr_10->gen_sym(bldr_10, "@test_fnc_v1.blk0");
    id_98 = bldr_10->gen_sym(bldr_10, "@test_fnc_v1.blk0.res");
    id_99 = bldr_10->gen_sym(bldr_10, NULL);
    bldr_10->new_binop(bldr_10, id_99, id_98, MU_BINOP_ASHR, id_91, id_92, id_93, MU_NO_ID);
    id_100 = bldr_10->gen_sym(bldr_10, NULL);
    bldr_10->new_ret(bldr_10, id_100, (MuVarNode [1]){id_98}, 1);
    bldr_10->new_bb(bldr_10, id_97, NULL, NULL, 0, MU_NO_ID, (MuInstNode [2]){id_99, id_100}, 2);
    bldr_10->new_func_ver(bldr_10, id_96, id_95, (MuBBNode [1]){id_97}, 1);
    bldr_10->load(bldr_10);
    mu_10->compile_to_sharedlib(mu_10, LIB_FILE_NAME("test_ashr"), NULL, 0);
    return 0;
}
