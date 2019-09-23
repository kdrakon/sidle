SHA_COMMAND ?= shasum -a 512
RUST_TARGET ?= x86_64-apple-darwin

clean:
	cargo clean

test: clean
	cargo test

build: clean
	rustup target add ${RUST_TARGET}
	cargo build -v --release --target ${RUST_TARGET}
	strip target/${RUST_TARGET}/release/sidle
	file target/${RUST_TARGET}/release/sidle

package: build
	cd target/${RUST_TARGET}/release/ && ${SHA_COMMAND} sidle > checksum-sha512
	tar vczf sidle_${RUST_TARGET}_${OS_TARGET}.tar.gz -C target/${RUST_TARGET}/release sidle checksum-sha512

