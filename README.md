# fqlib

A package written in Python for manipulating Illumina generated FastQ files. This code
is based on best practices developed and maintained at St. Jude Children's Research
Hospital. We only consider files which meet the following criteria:

* FastQ files (no support for Fasta)
* Illumina generated
* "Lazy" FastQ file structure (4 lines per read)

Anyone is welcome to contribute code or create issues.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.

If you are only interested in the command line tools, you can run the following
commands to get access to them:

```bash
pip install fqlib
fqlint
```

### Prerequisites

* Insure that Python 3.6+ is installed on your machine. `fqlib` may work with earlier versions of Python, but that would be coincidental.

### Installing

To get a full install of `fqlib` on your machine, you can run the following commands:

```bash
git clone https://github.com/stjude/fqlib.git
cd fqlib
pip install -r requirements.txt
python setup.py install
```

## Running the tests

No unit tests exist today, but we are working towards implementing them.

## Contributing

All contributions should be submitted to Github in the style of [Github Flow](https://guides.github.com/introduction/flow/index.html). All code must conform to
our [YAPF style configuration](.style.yapf) and must pass with pylint (you can see
our configuration [here](.pylintrc)). We recommend using [Visual Studio Code](https://code.visualstudio.com/) when coding for this project.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/stjude/fqlib/tags).

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
