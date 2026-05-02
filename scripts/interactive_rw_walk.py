#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import random
import subprocess
import sys
import time
from pathlib import Path


DEVICE_ORDER = [
    "STS10",
    "STC10",
    "STN10",
    "TS10",
    "TC10",
    "TN10",
    "CS10",
    "CC10",
    "CN10",
    "SB10",
    "SW10",
    "DX10",
    "DY10",
    "ZR10",
    "X10",
    "Y10",
    "M10",
    "L10",
    "F100",
    "V10",
    "B10",
    "D10",
    "W10",
    "Z10",
    "R10",
    "SM10",
    "SD10",
    "RD10",
    "LTS10",
    "LTC10",
    "LSTS10",
    "LSTC10",
    "LCS10",
    "LCC10",
    "LTN10",
    "LSTN10",
    "LCN10",
    "LZ0",
    "LZ1",
    r"J1\X10",
    r"J1\Y10",
    r"J1\B10",
    r"J1\W10",
    r"J1\SB10",
    r"J1\SW10",
]

PREFIX_ORDER = [
    "LSTS",
    "LSTC",
    "LSTN",
    "LTS",
    "LTC",
    "LTN",
    "STS",
    "STC",
    "STN",
    "SM",
    "SD",
    "TS",
    "TC",
    "TN",
    "CS",
    "CC",
    "CN",
    "SB",
    "SW",
    "DX",
    "DY",
    "LCS",
    "LCC",
    "LCN",
    "LZ",
    "ZR",
    "RD",
    "HG",
    "X",
    "Y",
    "M",
    "L",
    "F",
    "V",
    "B",
    "D",
    "W",
    "Z",
    "R",
    "G",
]

BIT_PREFIXES = {
    "STS",
    "STC",
    "TS",
    "TC",
    "CS",
    "CC",
    "SB",
    "DX",
    "DY",
    "X",
    "Y",
    "M",
    "L",
    "F",
    "V",
    "B",
    "SM",
    "LTS",
    "LTC",
    "LSTS",
    "LSTC",
    "LCS",
    "LCC",
}

DWORD_PREFIXES = {"LTN", "LSTN", "LCN", "LZ"}


def parse_args() -> argparse.Namespace:
    repo_root = Path(__file__).resolve().parents[1]
    default_bin = repo_root / "target" / "debug" / "slmp_verify_client"
    parser = argparse.ArgumentParser(
        description="Interactive read/write walk for PLC monitoring."
    )
    parser.add_argument("--host", default="192.168.250.100")
    parser.add_argument("--port", type=int, default=1025)
    parser.add_argument("--frame", default="4e", choices=["3e", "4e"])
    parser.add_argument("--series", default="iqr", choices=["legacy", "iqr"])
    parser.add_argument(
        "--family",
        default="iq-r",
        choices=["iq-f", "iq-r", "iq-l", "mx-f", "mx-r", "qcpu", "lcpu", "qnu", "qnudv"],
    )
    parser.add_argument("--seed", type=int, default=20260413)
    parser.add_argument("--start-at", default="")
    parser.add_argument("--client-bin", default=str(default_bin))
    parser.add_argument("--retries", type=int, default=4)
    parser.add_argument("--retry-delay-ms", type=int, default=500)
    parser.add_argument("--request-delay-ms", type=int, default=250)
    return parser.parse_args()


def split_extended(address: str) -> tuple[bool, str]:
    if "\\" in address:
        return True, address.split("\\", 1)[1]
    if "/" in address:
        return True, address.split("/", 1)[1]
    return False, address


def device_prefix(address: str) -> str:
    _, base = split_extended(address.upper())
    for prefix in PREFIX_ORDER:
        if base.startswith(prefix):
            return prefix
    raise ValueError(f"Unsupported device prefix: {address}")


def device_kind(address: str) -> str:
    prefix = device_prefix(address)
    if prefix in BIT_PREFIXES:
        return "bit"
    if prefix in DWORD_PREFIXES:
        return "dword"
    return "word"


def run_json(
    retries: int,
    retry_delay_ms: int,
    request_delay_ms: int,
    client_bin: str,
    host: str,
    port: int,
    frame: str,
    series: str,
    family: str,
    command: str,
    address: str,
    *extras: str,
    mode: str | None = None,
) -> dict:
    cmd = [client_bin, host, str(port), command, address]
    cmd.extend(str(item) for item in extras)
    if mode:
        cmd.extend(["--mode", mode])
    cmd.extend(["--frame", frame, "--series", series, "--family", family])
    last_error = None
    for attempt in range(retries + 1):
        proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
        combined = "\n".join(part for part in [proc.stdout, proc.stderr] if part).strip()
        last_line = ""
        for line in reversed(combined.splitlines()):
            if line.strip():
                last_line = line.strip()
                break
        if not last_line:
            last_error = RuntimeError(f"No output from client: {' '.join(cmd)}")
        else:
            try:
                payload = json.loads(last_line)
            except json.JSONDecodeError as exc:
                last_error = RuntimeError(
                    f"Unexpected client output: {last_line}\nfull_output:\n{combined}"
                )
            else:
                message = str(payload.get("message", "")).lower()
                retryable = (
                    payload.get("status") == "error"
                    and (
                        "connection refused" in message
                        or "timed out" in message
                        or "os error 111" in message
                    )
                )
                if not retryable:
                    if request_delay_ms > 0:
                        time.sleep(request_delay_ms / 1000.0)
                    return payload
                last_error = RuntimeError(
                    f"retryable transport error on attempt {attempt + 1}: {payload.get('message')}"
                )
        if attempt < retries and retry_delay_ms > 0:
            time.sleep(retry_delay_ms / 1000.0)
    raise last_error if last_error else RuntimeError(f"Client command failed: {' '.join(cmd)}")


def extract_plain_value(payload: dict):
    values = payload.get("values") or []
    return values[0] if values else None


def extract_ext_value(payload: dict):
    values = payload.get("values") or []
    if not values:
        return None
    return values[0]


def read_value(args: argparse.Namespace, address: str):
    extended, _ = split_extended(address)
    kind = device_kind(address)
    if extended:
        payload = run_json(
            args.retries,
            args.retry_delay_ms,
            args.request_delay_ms,
            args.client_bin,
            args.host,
            args.port,
            args.frame,
            args.series,
            args.family,
            "read-ext",
            address,
            "1",
            mode="bit" if kind == "bit" else None,
        )
        return payload["status"], extract_ext_value(payload), payload

    payload = run_json(
        args.retries,
        args.retry_delay_ms,
        args.request_delay_ms,
        args.client_bin,
        args.host,
        args.port,
        args.frame,
        args.series,
        args.family,
        "read-named",
        address,
    )
    return payload["status"], extract_plain_value(payload), payload


def write_value(args: argparse.Namespace, address: str, value: int) -> dict:
    extended, _ = split_extended(address)
    kind = device_kind(address)
    if extended:
        return run_json(
            args.retries,
            args.retry_delay_ms,
            args.request_delay_ms,
            args.client_bin,
            args.host,
            args.port,
            args.frame,
            args.series,
            args.family,
            "write-ext",
            address,
            str(value),
            mode="bit" if kind == "bit" else None,
        )

    if kind == "dword":
        return run_json(
            args.retries,
            args.retry_delay_ms,
            args.request_delay_ms,
            args.client_bin,
            args.host,
            args.port,
            args.frame,
            args.series,
            args.family,
            "random-write-words",
            "",
            "--dwords",
            f"{address}={value}",
        )

    return run_json(
        args.retries,
        args.retry_delay_ms,
        args.request_delay_ms,
        args.client_bin,
        args.host,
        args.port,
        args.frame,
        args.series,
        args.family,
        "write-named",
        f"{address}={value}",
    )


def write_status_ok(payload: dict) -> bool:
    return payload.get("status") == "success"


def format_status(payload: dict) -> str:
    if payload.get("status") == "success":
        return "success"
    return f"error: {payload.get('message', payload)}"


def iter_write_values(kind: str, rng: random.Random) -> list[int]:
    if kind == "bit":
        return [1, 0, 1, 0]
    if kind == "dword":
        return [rng.randrange(0, 0x7FFF_FFFF), rng.randrange(0, 0x7FFF_FFFF)]
    return [rng.randrange(0, 0x10000), rng.randrange(0, 0x10000)]


def wait_for_user(index: int, total: int, address: str) -> bool:
    reply = input(
        f"[{index}/{total}] {address} complete. Enter=next, q=quit > "
    ).strip().lower()
    return reply not in {"q", "quit", "exit"}


def main() -> int:
    args = parse_args()
    client_bin = Path(args.client_bin)
    if not client_bin.exists():
        print(f"Client binary not found: {client_bin}", file=sys.stderr)
        print(
            "Build it first with: cargo build --bin slmp_verify_client",
            file=sys.stderr,
        )
        return 1

    rng = random.Random(args.seed)
    print(
        f"interactive_rw_walk host={args.host}:{args.port} frame={args.frame} "
        f"series={args.series} family={args.family} seed={args.seed} "
        f"retries={args.retries} retry_delay_ms={args.retry_delay_ms} "
        f"request_delay_ms={args.request_delay_ms}"
    )
    print("Writes are not restored automatically. Monitor the PLC before continuing.")
    print()

    started = not args.start_at
    total = len(DEVICE_ORDER)

    for index, address in enumerate(DEVICE_ORDER, start=1):
        if not started:
            if address == args.start_at:
                started = True
            else:
                continue

        kind = device_kind(address)
        if args.family == "iq-f" and device_prefix(address) in {"DX", "DY"}:
            print("=" * 72)
            print(f"[{index}/{total}] {address} skipped: not supported for plc_family iq-f")
            continue

        extended, _ = split_extended(address)
        print("=" * 72)
        print(
            f"[{index}/{total}] {address} kind={kind} "
            f"path={'extended' if extended else 'plain'}"
        )

        try:
            status, current_value, read_payload = read_value(args, address)
            print(f"  read-before: {status} value={current_value!r}")
            if status != "success":
                print(f"  read-before payload: {read_payload}")
        except Exception as exc:  # noqa: BLE001
            print(f"  read-before failed: {exc}")

        for step_index, value in enumerate(iter_write_values(kind, rng), start=1):
            try:
                write_payload = write_value(args, address, value)
                print(
                    f"  write#{step_index}: value={value} -> {format_status(write_payload)}"
                )
                if not write_status_ok(write_payload):
                    break
                status, current_value, _ = read_value(args, address)
                print(
                    f"  read-back#{step_index}: {status} value={current_value!r}"
                )
                if status != "success":
                    break
            except Exception as exc:  # noqa: BLE001
                print(f"  step#{step_index} failed: {exc}")
                break

        print()
        if not wait_for_user(index, total, address):
            print("Stopped by user.")
            return 0

    print("All addresses processed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
