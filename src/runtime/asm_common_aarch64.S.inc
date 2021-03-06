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

// This file provides a template so that we can define functions in ASM and also
// let the symbols be recognised in both Linux and OSX

#if defined (__linux__)
  .macro begin_func n
  .text
  .globl \n
  .type \n,@function
  .balign 4
\n:
  .endm

  .macro end_func n
  .size \n, .-\n
  .endm

  .macro global_label n
  .globl \n
\n:
  .endm
#else
  #error "Only Linux is supported."
#endif

FP .req X29
LR .req X30

.macro push_pair src1 src2 stack=SP
    STP \src2 , \src1, [ \stack ,#-16]!
.endm

.macro pop_pair dest1 dest2 stack=SP
    LDP \dest1 , \dest2 , [ \stack ],#16
.endm

.macro enter_frame
push_pair LR, FP
MOV FP, SP
.endm

.macro exit_frame
    pop_pair FP, LR
.endm

.macro push_callee_saved stack=SP
    push_pair X19, X20, \stack
    push_pair X21, X22, \stack
    push_pair X23, X24, \stack
    push_pair X25, X26, \stack
    push_pair X27, X28, \stack

    push_pair D8, D9, \stack
    push_pair D10, D11, \stack
    push_pair D12, D13, \stack
    push_pair D14, D15, \stack
.endm

.macro pop_callee_saved stack=SP
    pop_pair D15, D14, \stack
    pop_pair D13, D12, \stack
    pop_pair D11, D10, \stack
    pop_pair D9, D8, \stack

    pop_pair X28, X27, \stack
    pop_pair X26, X25, \stack
    pop_pair X24, X23, \stack
    pop_pair X22, X21, \stack
    pop_pair X20, X19, \stack
.endm