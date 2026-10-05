# Benchmarks

From the repository root:

```sh
python3 -m venv target/bench-venv
source target/bench-venv/bin/activate
python3 -m pip install -r benches/requirements.txt
python3 benches/run.py
```

Results and plots are saved to `target/benchmarks/`.
