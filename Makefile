.PHONY: all fontbuild clean

all: fontbuild

fontbuild:
	cargo build --release

clean:
	rm -rf target/release
