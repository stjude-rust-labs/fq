# pylint: disable=all
from setuptools import setup

with open("README.md", "r") as f:
    long_description = f.read()

setup(
    name="fqlib",
    version="1.0.0",
    description=
    "A package written in Python for manipulating Illumina generated FastQ files.",
    license="MIT",
    long_description=long_description,
    author="Clay McLeod",
    author_email="clay.mcleod@stjude.org",
    url="https://github.com/stjude/fqlib",
    packages=["fqlib"],
    install_requires=[],
    scripts=["bin/fqlint"]
)