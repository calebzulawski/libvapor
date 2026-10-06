# Benchmarks

From the repository root:

```sh
python3 -m venv target/bench-venv
source target/bench-venv/bin/activate
python3 -m pip install -r benches/requirements.txt
python3 benches/run.py
```

Results and plots are saved to `target/benchmarks/`.

## WASM

Install [Wasmtime](https://docs.wasmtime.dev/cli-install.html) and put `wasmtime`
on `PATH`, then run the same benchmark harness through WASI:

```sh
rustup target add --toolchain nightly wasm32-wasip1
python3 benches/run.py --target wasm32-wasip1
```
