ifeq ($(origin MANIFEST_PATH), undefined)
else
  MANIFEST_PATH_ARGS = --manifest-path=${MANIFEST_PATH}
endif

ifeq ($(origin CLIPPY_FIX), undefined)
  CLIPPY_OPTS = --all-targets --no-deps
else
  CLIPPY_OPTS = --fix
endif

IGNORE_ARGS ?=

audit:
	cargo audit -D warnings

check-typos:
	typos

SET_LOWER_BOUNDS ?=
test-set-lower-bounds:
	echo "Testing that SET_LOWER_BOUNDS environment variable is set to a valid path"
	test -e "${SET_LOWER_BOUNDS}"

verify-dependency-bounds: test-set-lower-bounds
	cargo build ${MANIFEST_PATH_ARGS}
	${SET_LOWER_BOUNDS} ${MANIFEST_PATH_ARGS}
	cargo build ${MANIFEST_PATH_ARGS}

fmt:
	cargo fmt

fmt-ci:
	cargo fmt -- --check

build:
	cargo build

build-tests:
	cargo test --no-run

test:
	RUST_BACKTRACE=1 cargo test -- --skip sudo_ --skip loop_

sudo_test:
	RUST_BACKTRACE=1 RUST_TEST_THREADS=1 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E' cargo test

clippy:
	cargo clippy --all-features ${CLIPPY_OPTS}

docs:
	cargo doc --no-deps

yamllint:
	yamllint --strict .github/workflows/*.yml

.PHONY:
	audit
	build
	check-typos
	clippy
	docs
	fmt
	fmt-ci
	sudo_test
	test
	test-set-lower-bounds
	verify-dependency-bounds
	yamllint
