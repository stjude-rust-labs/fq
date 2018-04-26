SRCDIR=./fqlib
BUILDDIR=./build
DISTDIR=./dist
CFLAGS=""
PXD_FILES=$(wildcard $(SRCDIR)/*.pxd)
PYX_FILES=$(wildcard $(SRCDIR)/*.pyx)
HTML_FILES=$(PYX_FILES:pyx=html)
CLEANABLE=$(SRCDIR)/*.c $(SRCDIR)/*.cpp \
		  $(SRCDIR)/*.html $(SRCDIR)/*.so $(SRCDIR)/__pycache__ \
		  $(BUILDDIR)/* $(DISTDIR)/*
			
all:
	@echo "== Makefile Subcommands =="
	@echo ""
	@echo "  annotate: Create HTML files for all files, open browser."
	@echo "  build:    Build a production ready executable." 
	@echo "  clean:    Clean all non-essential files."
	@echo "  develop:  Compile package and symlink into local Python environment."
	@echo "  install:  Install the package to your local Python environment."
	@echo "  test:     Run the unit tests."
	@echo ""

%.html: %.pyx
	cd $(SRCDIR); cython -I ../ -a $<

.PHONY: build
build:
	CFLAGS="$(CFLAGS)" python setup.py build_ext

.PHONY: develop 
develop:
	CFLAGS="$(CFLAGS)" python setup.py develop

.PHONY: clean
clean:
	rm -rf $(CLEANABLE)

.PHONY: annotate
annotate: $(HTML_FILES)
	cd $(SRCDIR); python -m http.server

.PHONY: test
test:
	nosetests

# because I'm lazy and sometimes type tests
.PHONY: tests
tests: test
