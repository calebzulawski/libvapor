#!/usr/bin/env python3
"""Generate a speedup CSV and chart from benchmark logs, or plot an existing CSV."""

import argparse
import csv
import json
import math
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator

OPERATIONS = [
    "trunc", "floor", "ceil", "round", "fmod", "remainder",
    "sqrt", "cbrt", "hypot", "fma",
    "exp", "exp2", "expm1", "pow", "log", "log2", "log10", "log1p",
    "sin", "cos", "sincos", "tan", "asin", "acos", "atan", "atan2",
    "sinh", "cosh", "tanh", "asinh", "acosh", "atanh",
    "erf", "erfc", "lgamma", "tgamma",
]
COLORS = ["#4477AA", "#66CCEE", "#228833", "#CCBB44"]
KINDS = ["f32", "f64"]


def read_logs(directory, metadata):
    rows = []
    for build in metadata["builds"]:
        profile = build["profile"]
        log = directory / f"{profile.removeprefix('x86-64-')}.jsonl"
        events = [json.loads(line) for line in log.read_text().splitlines()]
        if not events or events[-1].get("event") != "ok":
            raise ValueError(f"Incomplete or failed {profile} results")
        measurements = [event for event in events if event.get("type") == "bench"]
        if len({event["name"] for event in measurements}) != len(measurements):
            raise ValueError(f"Duplicate {profile} measurements")
        pairs = {}
        for event in measurements:
            name = event["name"]
            stem, variant = name.split("::")
            operation, width = stem.rsplit("_", 1)
            kind, lanes = width.split("x")
            pairs.setdefault((operation, kind, int(lanes)), {})[variant] = event
        for (operation, kind, lanes), pair in sorted(pairs.items()):
            scalar, vector = pair["scalar"], pair["vector"]
            count = 32 * lanes
            rows.append({
                "profile": profile, "operation": operation, "kind": kind, "lanes": lanes,
                "elements_per_iteration": count,
                "scalar_ns_per_element": scalar["median"] / count,
                "vector_ns_per_element": vector["median"] / count,
                "speedup": scalar["median"] / vector["median"],
                # libtest's deviation is max minus min, not a confidence interval.
                "scalar_range_ns_per_element": scalar["deviation"] / count,
                "vector_range_ns_per_element": vector["deviation"] / count,
            })
    return rows


def load_results(results, output):
    source = results if results.is_dir() else results.parent
    metadata_path = source / "metadata.json"
    metadata = json.loads(metadata_path.read_text()) if metadata_path.exists() else {}
    if results.is_dir():
        rows = read_logs(source, metadata)
        if rows:
            with (output / "results.csv").open("w", newline="") as file:
                writer = csv.DictWriter(file, fieldnames=list(rows[0]))
                writer.writeheader()
                writer.writerows(rows)
    else:
        with results.open() as file:
            rows = list(csv.DictReader(file))
    return rows, metadata


def prepare_data(rows):
    profiles = sorted({row["profile"] for row in rows})
    present = {row["operation"] for row in rows}
    operations = [operation for operation in OPERATIONS if operation in present]
    operations += sorted(present - set(OPERATIONS))
    values = {(row["kind"], row["profile"], row["operation"]): float(row["speedup"]) for row in rows}
    if len(values) != len(rows):
        raise ValueError("Duplicate measurements.")
    if any(not math.isfinite(value) or value <= 0 for value in values.values()):
        raise ValueError("Speedups must be finite and positive.")
    lanes = {}
    for kind in KINDS:
        widths = {row["lanes"] for row in rows if row["kind"] == kind}
        if len(widths) != 1:
            raise ValueError("Compare one fixed vector width per precision.")
        lanes[kind] = widths.pop()
    return profiles, operations, values, lanes


def create_figure(profiles, operations):
    plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 10, "svg.fonttype": "none"})
    # Reserve a little vertical space for every value label, even for equal bars.
    figure_height = max(6, 0.15 * len(profiles) * len(operations) + 3.5)
    fig, (left, names, right) = plt.subplots(
        1, 3, figsize=(18, figure_height), sharey=True,
        gridspec_kw={"width_ratios": [1, 0.12, 1]},
    )
    fig.subplots_adjust(left=0.045, right=0.955, top=1 - 2.1 / figure_height,
                        bottom=0.95 / figure_height, wspace=0.015)
    fig.set_facecolor("#FAFBFD")
    for ax in (left, names, right):
        ax.set_facecolor("#FAFBFD")
        for index in range(0, len(operations), 2):
            ax.axhspan(index - 0.48, index + 0.48, color="#EEF2F7", zorder=0)
        ax.set_yticks([])
        ax.tick_params(length=0)
        for spine in ax.spines.values():
            spine.set_visible(False)
    left.set_ylim(len(operations) - 0.1, -1.1)
    names.set_xlim(0, 1)
    names.set_xticks([])
    for index, operation in enumerate(operations):
        names.text(0.5, index, operation, ha="center", va="center", fontsize=10, color="#172B4D")
    return fig, (left, right)


def draw_panels(axes, profiles, operations, values, lanes):
    limit = max(values.values()) * 1.15
    height = 0.78 / len(profiles)
    for kind, ax in zip(KINDS, axes):
        # Zero sits beside the names; larger speedups extend outward.
        ax.set_xlim((limit, 0) if kind == "f32" else (0, limit))
        for i, profile in enumerate(profiles):
            y = [index - 0.39 + height * (i + 0.5) for index in range(len(operations))]
            color = int(profile[-1]) - 1 if profile.startswith("x86-64-v") else i
            bars = ax.barh(y, [values[kind, profile, op] for op in operations], height=height * 0.92,
                           color=COLORS[color % len(COLORS)], label=profile, zorder=3)
            labels = ax.bar_label(bars, labels=[f"{values[kind, profile, op]:.2f}×" for op in operations],
                                  padding=3, fontsize=8, zorder=5)
            # Keep the 1× marker from crossing the numbers for slower results.
            for index, label in enumerate(labels):
                label.set_bbox({"facecolor": "#EEF2F7" if index % 2 == 0 else "#FAFBFD",
                                "edgecolor": "none", "pad": 0})
        ax.axvline(1, color="#334155", linestyle="--", linewidth=1, zorder=4)
        ax.annotate("Rust (1×)", xy=(1, 1), xycoords=("data", "axes fraction"),
                    xytext=(-2 if kind == "f32" else 2, 4), textcoords="offset points",
                    ha="right" if kind == "f32" else "left", va="bottom", fontsize=8,
                    color="#526175", annotation_clip=False)
        ax.xaxis.set_major_locator(MaxNLocator(8, steps=[1, 2, 2.5, 5, 10]))
        ax.grid(axis="x", color="#DCE2EA", linewidth=0.7, zorder=0)
        direction = "←" if kind == "f32" else "→"
        ax.set_title(f"{kind} × {lanes[kind]}  {direction}", fontsize=15, fontweight="bold",
                     color="#172B4D", pad=12)


def label_figure(fig, left, profiles, metadata):
    figure_height = fig.get_figheight()
    target = metadata.get("target", metadata.get("host", ""))
    title = "mwise vs plain Rust" + (f" · {target}" if target else "")
    fig.text(0.045, 1 - 0.45 / figure_height, title, fontsize=21,
             fontweight="bold", color="#172B4D")
    environment = [metadata.get("cpu"), metadata.get("os", metadata.get("system")),
                   metadata.get("runtime", "").split(" (", 1)[0]]
    environment = " · ".join(value for value in environment if value)
    fig.text(0.045, 1 - 0.83 / figure_height, environment or "Benchmark results",
             fontsize=11, color="#526175")
    handles, labels = left.get_legend_handles_labels()
    fig.legend(handles, labels, loc="upper center", bbox_to_anchor=(0.5, 1 - 1.1 / figure_height),
               ncols=len(profiles), frameon=False, fontsize=11, handlelength=1.4)
    notes = metadata.get("notes", "")
    if notes:
        fig.text(0.5, 1 - 1.6 / figure_height, notes,
                 ha="center", fontsize=9, color="#526175")
    baseline_time = "Rust time with auto-vectorization" if metadata.get("auto_vectorization") is True else "Rust time"
    fig.text(0.5, 0.4 / figure_height, f"Speedup (median {baseline_time} / median mwise time)",
             ha="center", fontsize=12, color="#172B4D")


def save_figure(fig, output):
    for extension in ["png", "svg"]:
        path = output / f"speedup.{extension}"
        fig.savefig(path, dpi=180, bbox_inches="tight", facecolor=fig.get_facecolor())
        print(path)
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", type=Path, nargs="?",
                        default=Path(__file__).resolve().parent.parent / "target/benchmarks")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    source = args.results if args.results.is_dir() else args.results.parent
    output = args.output or source
    output.mkdir(parents=True, exist_ok=True)
    rows, metadata = load_results(args.results, output)
    try:
        profiles, operations, values, lanes = prepare_data(rows)
    except ValueError as error:
        parser.error(str(error))
    fig, axes = create_figure(profiles, operations)
    draw_panels(axes, profiles, operations, values, lanes)
    label_figure(fig, axes[0], profiles, metadata)
    save_figure(fig, output)


if __name__ == "__main__":
    main()
