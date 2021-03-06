# Copyright 2017 The Australian National University
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""
Parse the muapi.h so that you can generate different bindings.

The result will be a simple JSON object (dict of dicts).
"""

import re

import injecttools

rfrag_commpragma = r'(?:///\s*MUAPIPARSER\s+(?P<pragma>.*))?'
r_comment = re.compile(r'//.*$', re.MULTILINE)
r_decl = re.compile(r'(?P<ret>\w+\s*\*?)\s*\(\s*\*\s*(?P<name>\w+)\s*\)\s*\((?P<params>[^)]*)\)\s*;\s*' + rfrag_commpragma, re.MULTILINE)
r_param = re.compile(r'\s*(?P<type>\w+\s*\*?)\s*(?P<name>\w+)')

r_define = re.compile(r'^\s*#define\s+(?P<name>\w+)\s*\(\((?P<type>\w+)\)(?P<value>\w+)\)\s*' + rfrag_commpragma + r'\s*$', re.MULTILINE)

r_typedef = re.compile(r'^\s*typedef\s+(?P<expand_to>\w+\s*\*?)\s*(?P<name>\w+)\s*;', re.MULTILINE)

r_struct_start = re.compile(r'^struct\s+(\w+)\s*\{')
r_struct_end = re.compile(r'^\};')

def filter_ret_ty(text):
    return text.replace(" ","")

def extract_params(text):
    params = []
    for text1 in text.split(','):
        ty, name = r_param.search(text1).groups()
        ty = ty.replace(" ",'')
        params.append({"type": ty, "name": name})

    return params

def extract_pragmas(text):
    text = text.strip()
    if len(text) == 0:
        return []
    else:
        return text.split(";")

def extract_method(name, params, ret_ty, pragmas):
    params = extract_params(params)
    ret_ty = filter_ret_ty(ret_ty)
    pragmas = extract_pragmas(pragmas)

    params_index = {p["name"]:p for p in params}

    for pragma in pragmas:
        parts = pragma.split(":")
        param_name = parts[0]
        if param_name not in params_index:
            raise Exception("Method {}: Pragma {} is for unknown param {}".format(
                name, pragma, param_name))

        param = params_index[param_name]

        kind = parts[1]
        if kind == 'array':
            sz_param_name = parts[2]
            param["array_sz_param"] = sz_param_name

            if sz_param_name not in params_index:
                raise Exception(
                        "Method {}: param {}: Array length parameter {} does not exist".format(
                            name, pn, sz_param_name))

            sz_param = params_index[sz_param_name]
            sz_param["is_sz_param"] = True

        elif kind == 'optional':
            param["is_optional"] = True
        elif kind == 'out':
            param["is_out"] = True
        else:
            raise Exception("Method {}: param {}: Unrecognised pragma {}".format(
                        name, pn, pragma))

    return {
            "name": name,
            "params": params,
            "ret_ty": ret_ty,
            # "pragmas": pragmas, # Don't include it any more, since we handle everything.
            }

def extract_methods(body):
    methods = []

    for ret, name, params, pragmas in r_decl.findall(body):
        method = extract_method(name, params, ret, pragmas)
        methods.append(method)
        
    return methods

def extract_struct(text, name):
    return injecttools.extract_lines(text, (r_struct_start, name), (r_struct_end,))

def extract_enum(name, ty, value, pragmas):
    pragmas = extract_pragmas(pragmas)
    enum = {
        "name": name,
        "value": value,
        # "pragmas": pragmas, # Don't include it any more, since we handle everything.
        }

    for pragma in pragmas:
        parts = pragma.split(":")
        pragma_name = parts[0]
        if pragma_name == "muname":
            muname = parts[1]
            enum["muname"] = muname
            
    return enum

def extract_enums(text, typename, pattern):
    defs = []
    for name, ty, value, pragmas in r_define.findall(text):
        if pattern.search(name) is not None:
            enum = extract_enum(name, ty, value, pragmas)
            defs.append(enum)
    return {
            "name": typename,
            "defs": defs,
            }

_top_level_structs = ["MuVM", "MuCtx", "MuIRBuilder"]
_enums = [(typename, re.compile(regex)) for typename, regex in [
    ("MuTrapHandlerResult", r'^MU_(THREAD|REBIND)'),
    ("MuDestKind",          r'^MU_DEST_'),
    ("MuBinOpStatus",       r'^MU_BOS_'),
    ("MuBinOptr",           r'^MU_BINOP_'),
    ("MuCmpOptr",           r'^MU_CMP_'),
    ("MuConvOptr",          r'^MU_CONV_'),
    ("MuMemOrd",            r'^MU_ORD_'),
    ("MuAtomicRMWOptr",     r'^MU_ARMW_'),
    ("MuCallConv",          r'^MU_CC_'),
    ("MuCommInst",          r'^MU_CI_'),
    ]]

def extract_typedefs(text):
    typedefs = {}
    typedefs_order = []
    for m in r_typedef.finditer(text):
        expand_to, name = m.groups()
        expand_to = expand_to.replace(" ","")
        typedefs[name] = expand_to
        typedefs_order.append((name, expand_to))

    return typedefs, typedefs_order

def parse_muapi(text):
    structs = []

    for sn in _top_level_structs:
        b = extract_struct(text, sn)
        methods = extract_methods(b)
        structs.append({"name": sn, "methods": methods})

    enums = []

    for tn,pat in _enums:
        enums.append(extract_enums(text, tn, pat))

    typedefs, typedefs_order = extract_typedefs(text)

    return {
            "structs": structs,
            "enums": enums,
            "typedefs": typedefs,
            "typedefs_order": typedefs_order,
            }

if __name__=='__main__':
    import sys, pprint, shutil

    width = 80

    try:
        width, height = shutil.get_terminal_size((80, 25))
    except:
        pass

    text = sys.stdin.read()
    pprint.pprint(parse_muapi(text), width=width)


