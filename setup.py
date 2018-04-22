# pylint: disable=all
# yapf: disable

import os
import sysconfig
from glob import glob
from setuptools import setup, Extension

try:
    from Cython.Build import cythonize
except:
    raise RuntimeError("You need Cython to build this package! Try 'pip install cython' first.")

# determine custom compiler flags
extra_compile_args = sysconfig.get_config_var('CFLAGS').split()
extra_compile_args += ["-std=c++0x", "-Wall", "-Wextra", "-lz"]
if 'CFLAGS' in os.environ:
    extra_compile_args += os.environ['CFLAGS'].split()

# build all extensions that need to be compiled dynamically.
extensions = []
base_path = os.path.dirname(os.path.realpath(__file__))
all_pyx = os.path.join(base_path, "fqlib/*.pyx")
for pyx in glob(all_pyx):

    stripped_filename = pyx.replace(base_path, "")
    if stripped_filename[0] == "/":
        stripped_filename = stripped_filename[1:]

    modulename = stripped_filename.replace(".pyx", "").replace(os.path.sep, ".")

    print(" [*] Adding extension for %s" % modulename)
    e = Extension(
        modulename,
        [stripped_filename],
        include_dirs=[base_path],
        language="c++",
        extra_compile_args=extra_compile_args
    )
    extensions.append(e)

with open("README.md", "r") as f:
    long_description = f.read()

setup(
    name="fqlib",
    version="1.0.2",
    python_requires='>3.6.1',
    description="A package written in Python for manipulating Illumina generated " \
                "FastQ files.",
    license="MIT",
    long_description=long_description,
    author="Clay McLeod",
    author_email="clay.mcleod@stjude.org",
    url="https://github.com/stjude/fqlib",
    packages=["fqlib"],
    scripts=["bin/fqlint", "bin/fqgen"],
    ext_modules=cythonize(extensions)
)
