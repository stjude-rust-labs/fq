default:
	CFLAGS="-lz" python setup.py develop

.phony: clean
clean:
	rm -rf fqlib/*.c fqlib/*.cpp fqlib/*.so fqlib/__pycache__ build/* dist/*
