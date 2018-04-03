import os
import shlex
from subprocess import Popen, PIPE


def command(cmd):
    process = Popen(shlex.split(cmd), stdout=PIPE, stderr=PIPE)
    (stdout, stderr) = process.communicate()
    exitcode = process.wait()
    return (stdout.decode("UTF-8"), stderr.decode("UTF-8"), exitcode)


def test_wellformed():
    (stdout, stderr, exitcode) = command(
        "fqlint example/00_wellformed/R1.fastq example/00_wellformed/R2.fastq"
    )
    assert exitcode == 0


def test_mismatched_read_names():
    (stdout, stderr, exitcode) = command(
        "fqlint example/01_mismatched_readnames/R1.fastq example/01_mismatched_readnames/R2.fastq"
    )
    assert exitcode == 2


def test_incomplete_reads():
    (stdout, stderr, exitcode) = command(
        "fqlint example/02_incomplete_reads/R1.fastq example/02_incomplete_reads/R2.fastq"
    )
    assert exitcode == 1