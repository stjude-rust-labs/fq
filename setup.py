# pylint: disable=all
# yapf: disable

import os
from setuptools import setup, Extension

try:
    from Cython.Build import cythonize
except:
    raise RuntimeError("You need Cython to build this package! Try 'pip install cython' first.")

with open("README.md", "r") as f:
    long_description = f.read()

os.environ['CFLAGS'] = '-O3 -Wall -std=c++11 -stdlib=libc++'
extensions = [
    Extension(
        "fqlib.timer",
        ["fqlib/timer.pyx"],
        include_dirs=["."],
        language="c++"
    ),
    Extension(
        "fqlib.error",
        ["fqlib/error.pyx"],
        include_dirs=["."],
        language="c++"
    ),
    Extension(
        "fqlib.utils",
        ["fqlib/utils.pyx"],
        include_dirs=["."],
        language="c++"
    ),
    Extension(
        "fqlib.validators",
        ["fqlib/validators.pyx"],
        include_dirs=["."],
        language="c++"
    ),
    Extension(
        "fqlib.fqread",
        ["fqlib/fqread.pyx"],
        include_dirs=["."],
        language="c++"
    ),
    Extension(
        "fqlib.fastq",
        ["fqlib/fastq.pyx"],
        include_dirs=["."],
        language="c++"
    )
]

setup(
    name="fqlib",
    version="1.0.1",
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
    ext_modules=cythonize(extensions),
#    package_data = {
#        "fqlib/*.pxd"
#    }
)
