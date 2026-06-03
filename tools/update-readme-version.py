#!/usr/bin/env python3
"""Set README dependency example to match Cargo.toml version (major.minor range)."""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CARGO = ROOT / "Cargo.toml"
README = ROOT / "README.md"


def main() -> None:
    cargo_text = CARGO.read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', cargo_text, re.MULTILINE)
    if not match:
        print("Could not find version in Cargo.toml", file=sys.stderr)
        sys.exit(1)

    version = match.group(1)
    parts = version.split(".")
    if len(parts) >= 2:
        semver_range = f"{parts[0]}.{parts[1]}"
    else:
        semver_range = version

    readme_text = README.read_text(encoding="utf-8")
    new_text, count = re.subn(
        r'^ens-normalize-rs = "[^"]+"$',
        f'ens-normalize-rs = "{semver_range}"',
        readme_text,
        count=1,
        flags=re.MULTILINE,
    )
    if count == 0:
        print('Could not find ens-normalize-rs = "..." line in README.md', file=sys.stderr)
        sys.exit(1)

    README.write_text(new_text, encoding="utf-8")
    print(f'README.md: ens-normalize-rs = "{semver_range}" (from Cargo.toml {version})')


if __name__ == "__main__":
    main()
