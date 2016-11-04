"""
Harness JIT tests using py.test framework
"""
import subprocess as subp
import os, sys
import ctypes
import py

CC = 'clang'
proj_dir = py.path.local(__file__).join('..', '..', '..')
test_jit_dir = proj_dir.join('tests', 'test_jit')
testsuite_dir = test_jit_dir.join('suite')
bin_dir = test_jit_dir.join('temp')
if not bin_dir.exists():
    bin_dir.mkdir()

def compile_lib(testname):
    src_c = testsuite_dir.join(testname + '.c')
    bin_path = bin_dir.join(testname)
    CFLAGS = [
        "-std=c99",
        "-I%(proj_dir)s/src/vm/api" % globals(),
        "-L%(proj_dir)s/target/debug" % globals(),
        "-lmu",
    ]
    cmd = [CC] + CFLAGS + ['-o', str(bin_path)] + [str(src_c)]

    # compile
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE, env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, cmd)
     
    os.environ['LD_LIBRARY_PATH'] = "%s:%s" % ("%(proj_dir)s/target/debug" % globals(),
                                               os.environ['LD_LIBRARY_PATH'] if 'LD_LIBRARY_PATH' in os.environ else "")
    # run
    p = subp.Popen([str(bin_path)], stdout=subp.PIPE, stderr=subp.PIPE, cwd=str(bin_dir), env=os.environ)
    out, err = p.communicate()
    if p.returncode != 0:  # failed
        sys.stdout.write(out + '\n')
        sys.stderr.write(err + '\n')
        raise subp.CalledProcessError(p.returncode, bin_path)

    return str(bin_dir.join('emit', testname + '.dylib'))

def get_fncptr(testname, entry_fnc, argtypes=[], restype=ctypes.c_ulonglong):
    lib_path = compile_lib(testname)
    lib = ctypes.CDLL(lib_path)
    fncptr = getattr(lib, entry_fnc)
    fncptr.restype = restype
    fncptr.argtypes = argtypes
    return fncptr

def test_constant_function():
    fn = get_fncptr("test_constfunc", 'test_fnc')
    assert fn() == 0

def test_milsum():
    fn = get_fncptr("test_milsum", "milsum", [ctypes.c_ulonglong])
    assert fn(1000000) == 500000500000

def test_factorial():
    fn = get_fncptr("test_fac", "fac", [ctypes.c_ulonglong])
    assert fn(20) == 2432902008176640000

def test_fibonacci():
    fn = get_fncptr("test_fib", "fib", [ctypes.c_ulonglong])
    assert fn(20) == 6765

def test_multifunc():
    fn = get_fncptr("test_multifunc", "entry")
    assert fn() == 6765