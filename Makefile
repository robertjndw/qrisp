WASM_TARGET := plugin/target/wasm32-unknown-unknown/release/qrisp_plugin.wasm

.PHONY: wasm test test-plugin test-e2e examples install clean

# Rebuild the plugin. The committed qrisp.wasm must match this output.
# -O3 rather than -Oz: Typst interprets wasm, so speed matters more than a few KB.
wasm:
	cargo build --manifest-path plugin/Cargo.toml --release --target wasm32-unknown-unknown
	wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
		$(WASM_TARGET) -o qrisp.wasm

test: wasm test-plugin test-e2e

# Unit tests plus tests that need the typst CLI on PATH (errors, e2e).
test-plugin:
	cargo test --manifest-path plugin/Cargo.toml --profile test-opt --lib --test codewords --test knockout --test errors

test-e2e:
	cargo test --manifest-path plugin/Cargo.toml --profile test-opt --test e2e

# Every example except the shared helper becomes a picture in docs/ for the README.
EXAMPLES := $(filter-out common,$(basename $(notdir $(wildcard examples/*.typ))))

examples: $(EXAMPLES:%=docs/%.png)

docs/%.png: examples/%.typ examples/common.typ examples/logo.svg qrisp.wasm $(wildcard src/*.typ)
	typst compile --root . --ppi 110 $< $@

# Install the working tree as @local/qrisp:<version>.
install:
	./install.sh

clean:
	cargo clean --manifest-path plugin/Cargo.toml
