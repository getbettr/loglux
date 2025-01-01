DEFAULT_RELEASE := "patch"
RUSTFLAGS := "-Zlocation-detail=none -Zfmt-debug=none"
CROSS_TARGET := "x86_64-unknown-linux-musl"

_default:
  @just --list

ci:
  #!/usr/bin/env bash
  export CARGO_PROFILE_RELEASE_LTO=false
  cargo clippy --locked -- -D warnings
  QUICKCHECK_TESTS="$((2**14))" RUST_LOG=quickcheck cargo test -- --nocapture --test-threads=1

release type=DEFAULT_RELEASE:
  #!/usr/bin/env bash
  cargo release version {{type}} -x --no-confirm
  cargo release commit -x --no-confirm
  cargo release -x --no-confirm --no-push || {
    echo -e "Release failed, removing latest tag and rewinding to HEAD~1..."
    git tag --delete $(git tag -l | sort -r | head -n 1)
    git reset --hard HEAD~1
    exit 1
  }

build: clean
  RUSTFLAGS="{{RUSTFLAGS}}" cargo +nightly build \
    -Z build-std=std,panic_abort \
    -Z build-std-features="panic_immediate_abort,optimize_for_size" \
    --target x86_64-unknown-linux-gnu --profile smol

bloat: clean
  RUSTFLAGS="{{RUSTFLAGS}}" cargo +nightly bloat \
    -Z build-std=std,panic_abort \
    -Z build-std-features="panic_immediate_abort,optimize_for_size" \
    --target x86_64-unknown-linux-gnu --profile smol

cross:
  #!/usr/bin/env bash
  if [[ ! -v CI ]] && test -f $PWD/.cross/Cross.toml; then
    export CROSS_CONFIG=$PWD/.cross/Cross.toml
    export CROSS_CONTAINER_OPTS="-v $HOME/.cache/sccache:/sccache -e SCCACHE_DIR=/sccache"
  fi
  env | grep CROSS_ | sort
  RUSTFLAGS="" cross build --release --target {{CROSS_TARGET}}

clean:
    cargo clean

fmt:
    cargo +nightly fmt
