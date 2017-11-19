INSTALL := install

PREFIX := /usr/local
BINDIR := $(PREFIX)/bin

.PHONY: all sss_fontbuild clean install

all: sss_fontbuild

sss_fontbuild:
	cargo build --release

clean:
	rm -rf target/release

install: fontbuild
	$(INSTALL) -d $(BINDIR)
	$(INSTALL) target/release/sss_fontbuild $(BINDIR)
