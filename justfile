book:
  pushd docs && mdbook build --open

serve:
  pushd docs && mdbook serve --open


run:
    cd code/hello_world && cargo run --release
