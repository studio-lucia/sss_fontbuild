INSTALL := install

PREFIX := /usr/local
BINDIR := $(PREFIX)/bin

.PHONY: all fontbuild clean install

all: fontbuild

fontbuild:
	cargo build --release

clean:
	rm -rf target/release

install: fontbuild
	$(INSTALL) -d $(BINDIR)
	$(INSTALL) fontbuild $(BINDIR)
