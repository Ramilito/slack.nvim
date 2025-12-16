.PHONY: llscheck luacheck stylua

llscheck:
	llscheck --configpath .luarc.json .

luacheck:
	luacheck lua

stylua:
	stylua --color always --check lua

.PHONY: check
check: llscheck luacheck stylua

.PHONY: clean
clean:
	cargo clean

.PHONY: build_dev
build_dev:
	cargo build --features telemetry

.PHONY: build_release
build_release:
	@cargo build --release --features telemetry

.PHONY: build
build:
	cargo build --release
