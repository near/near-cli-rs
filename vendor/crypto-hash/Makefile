CARGO ?= cargo
CARGO_BUILD_TEST = $(CARGO) test --no-run
KCOV ?= kcov
TEST_APP = debug/crypto_hash-*.exe
WIN_TARGET = x86_64-pc-windows-gnu

build-test:
	$(CARGO_BUILD_TEST)

check-i686:
	PKG_CONFIG_LIBDIR=/usr/lib/i386-linux-gnu/pkgconfig \
		PKG_CONFIG_ALLOW_CROSS=1 \
		$(CARGO) test --target i686-unknown-linux-gnu --verbose

check-wine64:
	$(CARGO_BUILD_TEST) --target $(WIN_TARGET)
	WINEPREFIX=$(HOME)/.local/share/wineprefixes/wine64 wine64 target/$(WIN_TARGET)/$(TEST_APP)

cov: build-test
	$(KCOV) --exclude-pattern=/.multirust,test.rs target/cov target/$(TEST_APP)

debug: build-test
	rust-gdb target/$(TEST_APP)

fmt:
	$(CARGO) fmt

lint:
	$(CARGO) +nightly clippy -- --allow clippy::pedantic
