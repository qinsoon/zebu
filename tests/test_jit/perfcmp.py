"""
Performance comparison
"""
from time import time
from tempfile import mkdtemp
import py, os, sys
import subprocess as subp
import ctypes
import math

from rpython.rtyper.lltypesystem import lltype, rffi
from rpython.translator.interactive import Translation
from rpython.config.translationoption import set_opt_level

from util import libext, preload_libmu

perf_target_dir = py.path.local(__file__).dirpath().join('perftarget')


CPYTHON = os.environ.get('CPYTHON', 'python')
PYPY = os.environ.get('PYPY', 'pypy')
RPYTHON = os.environ.get('RPYTHON', None)
CC = os.environ.get('CC', 'clang')


def run(cmd):
    print ' '.join(cmd)
    p = subp.Popen(cmd, stdout=subp.PIPE, stderr=subp.PIPE)
    return p.communicate()


def run_cpython(config):
    py_file = config['py_file']
    out, _ = run([CPYTHON, py_file.strpath] + map(str, config['setup_args']))
    return float(out)


def run_pypy_nojit(config):
    py_file = config['py_file']
    out, _ = run([PYPY, '--jit', 'off', py_file.strpath] + map(str, config['setup_args']))
    return float(out)


def run_pypy(config):
    py_file = config['py_file']
    out, _ = run([PYPY, py_file.strpath] + map(str, config['setup_args']))
    return float(out)

rpy_wrapper = \
"""
def rpy_measure_%(name)s_%(target)s(%(args)s):
    from time import time
    t0 = time()
    %(rpy_fnc)s(%(args)s)
    t1 = time()
    return t1 - t0
"""


def compile_rpython_c(config):
    target = 'rpy_c'
    rpyfnc = config['rpy_fnc']
    wrapper_config = {
        'name': rpyfnc.__name__ + target,
        'target': target,
        'rpy_fnc': rpyfnc.__name__,
        'args': ', '.join(['v%d' % i for i in range(len(config['llarg_ts']))])
    }
    tl_config = {'gc': 'none'}

    wrapper = rpy_wrapper % wrapper_config
    locals()[rpyfnc.__name__] = rpyfnc
    exec wrapper in locals()
    rpy_measure_fnc = locals()['rpy_measure_%(name)s_%(target)s' % wrapper_config]
    t = Translation(rpy_measure_fnc, config['llarg_ts'], **tl_config)
    set_opt_level(t.config, '3')
    libpath = t.compile_c()
    eci = rffi.ExternalCompilationInfo(libraries=[libpath.strpath])
    fnp = rffi.llexternal('pypy_g_' + rpy_measure_fnc.__name__, config['llarg_ts'], rffi.DOUBLE,
                          compilation_info=eci, _nowrapper=True)
    return fnp


def compile_rpython_mu(config):
    preload_libmu()

    target = 'rpy_mu'
    rpyfnc = config['rpy_fnc']
    wrapper_config = {
        'name': rpyfnc.__name__ + target,
        'target': target,
        'rpy_fnc': rpyfnc.__name__,
        'args': ', '.join(['v%d' % i for i in range(len(config['llarg_ts']))])
    }
    tl_config = {'backend': 'mu', 'muimpl': 'fast', 'mucodegen': 'api', 'mutestjit': True}

    wrapper = rpy_wrapper % wrapper_config
    locals()[rpyfnc.__name__] = rpyfnc
    exec wrapper in locals()
    rpy_measure_fnc = locals()['rpy_measure_%(name)s_%(target)s' % wrapper_config]
    t = Translation(rpy_measure_fnc, config['llarg_ts'], **tl_config)
    set_opt_level(t.config, '3')

    db, bdlgen, fnc_name = t.compile_mu()
    libpath = libpath = config['libpath_mu']
    bdlgen.mu.compile_to_sharedlib(libpath.strpath, [])
    eci = rffi.ExternalCompilationInfo(libraries=[libpath.strpath])
    fnp = rffi.llexternal(fnc_name, config['llarg_ts'], rffi.DOUBLE,
                          compilation_info=eci, _nowrapper=True)
    return fnp


def compile_c(config):
    libpath = config['libpath_c']
    c_fnc = rffi.llexternal(config['c_sym_name'], config['llarg_ts'], config['llres_t'],
                            compilation_info=rffi.ExternalCompilationInfo(
                                includes=['quicksort.h'],
                                include_dirs=[perf_target_dir.strpath],
                                separate_module_sources=['#include "quicksort.c"']
                            ), _nowrapper=True)

    target = 'c'
    rpyfnc = config['rpy_fnc']
    wrapper_config = {
        'name': rpyfnc.__name__ + target,
        'target': target,
        'rpy_fnc': 'c_fnc',
        'args': ', '.join(['v%d' % i for i in range(len(config['llarg_ts']))])
    }
    tl_config = {'gc': 'none'}

    wrapper = rpy_wrapper % wrapper_config
    exec wrapper in locals()
    rpy_measure_fnc = locals()['rpy_measure_%(name)s_%(target)s' % wrapper_config]
    t = Translation(rpy_measure_fnc, config['llarg_ts'], **tl_config)
    set_opt_level(t.config, '3')
    libpath = t.compile_c()
    eci = rffi.ExternalCompilationInfo(libraries=[libpath.strpath])
    fnp = rffi.llexternal('pypy_g_' + rpy_measure_fnc.__name__, config['llarg_ts'], rffi.DOUBLE,
                          compilation_info=eci, _nowrapper=True)
    return fnp


def get_stat(run_fnc, config, iterations=100):
    times = []
    for i in range(iterations):
        times.append(run_fnc(config))

    times.sort()
    avg = sum(times) / float(len(times))
    t_min = t_max = t_std = None
    if len(times) > 1:
        t_min = times[0]
        t_max = times[-1]
        squares = ((t - avg) ** 2 for t in times)
        t_std = math.sqrt(sum(squares) / (len(times) - 1))

    return {'average': avg, 't_min': t_min, 't_max': t_max, 'std_dev': t_std, 'data': times}


def get_stat_compiled(compile_fnc, config, iterations=100):
    def run_funcptr(fnp, config):
        args = config['setup'](*config['setup_args'])
        t = fnp(*args)
        config['teardown'](*args)
        return t

    fnp = compile_fnc(config)
    return get_stat(lambda config: run_funcptr(fnp, config), config, iterations)


def get_display_str(stat):
    output = "average: %(average)s\n" \
             "min: %(t_min)s\n" \
             "max: %(t_max)s\n" \
             "std_dev: %(std_dev)s\n"
    return output % stat


def perf(config, iterations):
    results = {
        # 'cpython': get_stat(run_cpython, config, iterations=iterations),
        # 'pypy_nojit': get_stat(run_pypy_nojit, config, iterations=iterations),
        # 'pypy': get_stat(run_pypy, config, iterations=iterations),
        'rpy_c': get_stat_compiled(compile_rpython_c, config, iterations=iterations),
        'rpy_mu': get_stat_compiled(compile_rpython_mu, config, iterations=iterations),
        'c': get_stat_compiled(compile_c, config, iterations=iterations),
    }

    for python, result in results.items():
        print '\033[35m---- %(python)s ----\033[0m' % locals()
        print get_display_str(result)

    return results


def save_results(test_name, results):
    import json
    json_file_path = py.path.local('result_%(test_name)s.json' % locals())

    with json_file_path.open('w') as fp:
        json.dump(results, fp, indent=4, separators=(',', ':'))


def perf_fibonacci(N, iterations):
    from perftarget.fibonacci import fib, rpy_entry
    tmpdir = py.path.local(mkdtemp())
    print tmpdir

    config = {
        'py_file': perf_target_dir.join('fibonacci.py'),
        'c_file': perf_target_dir.join('fibonacci.c'),
        'rpy_fnc': rpy_entry,
        'c_sym_name': 'fib',
        'llarg_ts': [lltype.Signed],
        'llres_t': lltype.Signed,
        'setup_args': (N,),
        'setup': lambda N: (N, ),
        'teardown': lambda N: None,
        'libpath_mu': tmpdir.join('libfibonacci_mu.dylib'),
        'libpath_c': tmpdir.join('libfibonacci_c.dylib')
    }

    results = perf(config, iterations)
    results['test_name'] = 'fibonacci'
    results['input_size'] = N
    results['iterations'] = iterations
    return results


def perf_arraysum(N, iterations):
    from perftarget.arraysum import arraysum, setup, teardown
    tmpdir = py.path.local(mkdtemp())
    print tmpdir

    config = {
        'py_file': perf_target_dir.join('arraysum.py'),
        'c_file': perf_target_dir.join('arraysum.c'),
        'rpy_fnc': arraysum,
        'c_sym_name': 'arraysum',
        'llarg_ts': [rffi.CArrayPtr(rffi.LONGLONG), rffi.SIZE_T],
        'llres_t': rffi.LONGLONG,
        'setup_args': (N, ),
        'setup': setup,
        'teardown': teardown,
        'libpath_mu': tmpdir.join('libfibonacci_mu.dylib'),
        'libpath_c': tmpdir.join('libfibonacci_c.dylib')
    }

    results = perf(config, iterations)
    results['test_name'] = 'arraysum'
    results['input_size'] = N
    results['iterations'] = iterations
    return results


def perf_quicksort(N, iterations):
    from perftarget.quicksort import quicksort, setup, teardown
    tmpdir = py.path.local(mkdtemp())
    print tmpdir
    import os
    os.environ['LIBRARY_PATH'] = tmpdir.strpath
    config = {
        'py_file': perf_target_dir.join('quicksort.py'),
        'c_file': perf_target_dir.join('quicksort.c'),
        'rpy_fnc': quicksort,
        'c_sym_name': 'quicksort',
        'llarg_ts': [rffi.CArrayPtr(rffi.LONGLONG), lltype.Signed, lltype.Signed],
        'llres_t': lltype.Void,
        'setup_args': (N,),
        'setup': setup,
        'teardown': teardown,
        'libpath_mu': tmpdir.join('libquicksort_mu' + libext),
        'libpath_c': tmpdir.join('libquicksort_c' + libext)
    }

    results = perf(config, iterations)
    results['test_name'] = 'quicksort'
    results['input_size'] = N
    results['iterations'] = iterations
    return results


def test_functional_fibonacci():
    save_results('fibonacci', perf_fibonacci(5, 1))


def test_functional_arraysum():
    save_results('arraysum', perf_arraysum(100, 1))


def test_functional_quicksort():
    save_results('quicksort', perf_quicksort(100, 5))


def plot(result_dic):
    import matplotlib.pyplot as plt
    fig = plt.figure(1, figsize=(9, 6))
    ax = fig.add_subplot(111)
    width = 0.1

    colors = ['#c82829',
              '#f5871f',
              '#eab700',
              '#718c00',
              '#3e999f',
              '#4271ae',
              '#8959a8',
              '#1d1f21']

    all_targets = ('cpython', 'pypy', 'pypy_nojit', 'rpy_c', 'rpy_mu', 'c')
    compiled_targets = ('rpy_c', 'rpy_mu', 'c')
    targets = compiled_targets
    data = [(tgt, result_dic[tgt]['average'], result_dic[tgt]['std_dev'])
            for tgt in targets]
    data.sort(key=lambda (tgt, avg, std): avg)
    for i, (tgt, avg, std_dev) in enumerate(data):
        ax.bar(width / 2 + width * i, avg, width, color=colors[i], yerr=std_dev, label=tgt)
        ax.text(width / 2 + width * i + 0.01, avg, "%.6f" % avg, color='#1d1f21', fontweight='bold')
        ax.text(width * (i + 1), avg - std_dev, "%.6f" % std_dev, color='#1d1f21', fontweight='bold')

    # plt.legend(loc=2)
    plt.xticks([width * (i + 1) for i in range(len(targets))], targets)
    plt.title("%(test_name)s with input size %(input_size)d" % result_dic)
    plt.show()


def test_plot():
    plot(perf_quicksort(1000, 20))
    # import json
    # with open('result_quicksort.json', 'r') as fp:
    #     dic = json.load(fp)
    # plot(dic)


if __name__ == '__main__':
    # fib_res = perf_fibonacci(40, 20)
    # save_results('fibonacci', fib_res)
    # arraysum_res = perf_arraysum(1000000, 20)
    # save_results('arraysum', arraysum_res)
    # quicksort_res = perf_quicksort(1000000, 20)
    # save_results('quicksort', quicksort_res)
    test_functional_quicksort()
