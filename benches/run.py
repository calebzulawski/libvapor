#!/usr/bin/env python3
"""Run the configured benchmark profiles and generate their chart."""

import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

from cpuinfo import get_cpu_info

BENCHES = Path(__file__).resolve().parent
ROOT = BENCHES.parent


def machine_description():
    cpu = get_cpu_info().get("brand_raw") or platform.processor() or platform.machine()
    if platform.system() == "Darwin":
        return cpu, f"macOS {platform.mac_ver()[0]}"
    system = platform.platform()
    if platform.system() == "Linux":
        system = platform.freedesktop_os_release()["PRETTY_NAME"]
    return cpu, system


def make_metadata(notes):
    cpu, system = machine_description()
    version = subprocess.check_output(["rustc", "+nightly", "--version"], text=True).strip()
    return {"cpu": cpu, "os": system, "kernel": platform.release(), "rustc": version,
            "auto_vectorization": True, "notes": notes, "builds": []}


def x86_cpu_level(output):
    """Query CPU and OS support without assuming optional x86 instructions."""
    env = {**os.environ, "CARGO_BUILD_TARGET": "host-tuple",
           "CARGO_TARGET_DIR": str(output / "cpu-probe-build"),
           "CARGO_ENCODED_RUSTFLAGS": "-Ctarget-cpu=x86-64"}
    return int(subprocess.check_output(
        ["cargo", "+nightly", "-Zscript", "--quiet", str(BENCHES / "support/cpu-levels.rs")],
        env=env, text=True))


def load_profiles(output, target):
    architecture = "x86_64" if platform.machine().lower() in {"x86_64", "amd64"} else "other"
    if target == "wasm32-wasip1":
        architecture = "wasm32"
    config = json.loads((BENCHES / "profiles.json").read_text())[architecture]
    profiles = config["profiles"]
    if architecture == "x86_64":
        cpu_level = x86_cpu_level(output)
        profiles = {name: flags for name, flags in profiles.items() if int(name[-1]) <= cpu_level}
    return profiles, config["notes"]


def run_profile(name, flags, output, cpu, target):
    print(f"Running {name}: {' '.join(flags)}", flush=True)
    env = {**os.environ, "RUSTFLAGS": " ".join(flags)}
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    command = ["cargo", "+nightly", "bench", "--locked", "--bench", "math", "--target", target,
               "--target-dir", str(output / "build")]
    if target == "wasm32-wasip1":
        env["CARGO_TARGET_WASM32_WASIP1_RUNNER"] = "wasmtime run -W relaxed-simd=y --"
    subprocess.run(command + ["--no-run"], cwd=ROOT, env=env, check=True)
    with (output / f"{name.removeprefix('x86-64-')}.jsonl").open("w") as log:
        subprocess.run(command + ["--", "--test-threads=1", "--format=json", "-Zunstable-options"],
                       cwd=ROOT, env=env, stdout=log, check=True,
                       preexec_fn=(lambda: os.sched_setaffinity(0, {cpu})) if cpu is not None else None)
    return {"profile": name, "rustflags": " ".join(flags)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=["host-tuple", "wasm32-wasip1"], default="host-tuple")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    directory = "benchmarks-wasm" if args.target == "wasm32-wasip1" else "benchmarks"
    output = (args.output or ROOT / "target" / directory).resolve()
    output.mkdir(parents=True, exist_ok=True)
    os.environ["PATH"] += os.pathsep + str(Path.home() / ".cargo/bin")
    profiles, notes = load_profiles(output, args.target)
    cpu = min(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None
    metadata = make_metadata(notes)
    metadata["target"] = args.target
    if args.target == "host-tuple":
        metadata["target"] = subprocess.check_output(
            ["rustc", "+nightly", "--print", "host-tuple"], text=True).strip()
    else:
        metadata["runtime"] = subprocess.check_output(["wasmtime", "--version"], text=True).strip()
    for name, flags in profiles.items():
        metadata["builds"].append(run_profile(name, flags, output, cpu, args.target))
        (output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    subprocess.run([sys.executable, str(BENCHES / "plot.py"), str(output)], check=True)


if __name__ == "__main__":
    main()
