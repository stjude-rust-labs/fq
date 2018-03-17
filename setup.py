# pylint: disable=all
# yapf: disable
from setuptools import setup
from Cython.Build import cythonize

with open("README.md", "r") as f:
    long_description = f.read()

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
    install_requires=["cython"],
    scripts=["bin/fqlint"],
    ext_modules=cythonize("fqlib/*.pyx")
)