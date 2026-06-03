#!/usr/bin/env python3
"""
Unicode normalization files update checker.

This script checks for updates to Unicode normalization files from the
ens-normalize.js repository and downloads them if changes are detected.
"""

import requests
import json
import hashlib
import os
import shutil
import sys
import argparse
from typing import Tuple, Dict, Any


def get_file_hash(file_path: str) -> str | None:
    """Calculate SHA256 hash of a file."""
    if not os.path.exists(file_path):
        return None
    with open(file_path, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def download_file(url: str, filename: str) -> str:
    """Download file and return its hash."""
    print(f"Downloading {url}...")
    response = requests.get(url, timeout=30)
    response.raise_for_status()
    
    with open(filename, 'wb') as f:
        f.write(response.content)
    
    return hashlib.sha256(response.content).hexdigest()


def _metadata_record(data: Any) -> Dict[str, Any]:
    """Normalize nf/spec objects or tests.json version entry into metadata fields."""
    if not isinstance(data, dict):
        return {}
    return {
        'created': data.get('created', data.get('validated', 'Unknown')),
        'unicode': data.get('unicode', 'Unknown'),
        'cldr': data.get('cldr', 'Unknown'),
    }


def get_file_metadata(file_path: str) -> Dict[str, Any]:
    """Extract metadata from JSON file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        if isinstance(data, list):
            version_entry = next(
                (item for item in data if isinstance(item, dict) and item.get('name') == 'version'),
                None,
            )
            data = version_entry if version_entry is not None else (data[0] if data else {})
        return _metadata_record(data)
    except (json.JSONDecodeError, KeyError, FileNotFoundError) as e:
        print(f"Warning: Could not read metadata from {file_path}: {e}")
        return {
            'created': 'Unknown',
            'unicode': 'Unknown',
            'cldr': 'Unknown'
        }


def check_for_updates(
    nf_url: str,
    spec_url: str,
    tests_url: str,
    nf_local: str,
    spec_local: str,
    tests_local: str,
    force_update: bool = False
) -> Tuple[bool, Dict[str, Any]]:
    """
    Check for updates to Unicode normalization files.
    
    Returns:
        Tuple of (should_update, metadata_dict)
    """
    print("Checking for Unicode normalization file updates...")
    
    # Get current hashes
    nf_current_hash = get_file_hash(nf_local)
    spec_current_hash = get_file_hash(spec_local)
    tests_current_hash = get_file_hash(tests_local)
    
    print(f"Current nf.json hash: {nf_current_hash}")
    print(f"Current spec.json hash: {spec_current_hash}")
    print(f"Current tests.json hash: {tests_current_hash}")
    
    # Download new files
    nf_new_hash = download_file(nf_url, "nf_new.json")
    spec_new_hash = download_file(spec_url, "spec_new.json")
    tests_new_hash = download_file(tests_url, "tests_new.json")
    
    print(f"New nf.json hash: {nf_new_hash}")
    print(f"New spec.json hash: {spec_new_hash}")
    print(f"New tests.json hash: {tests_new_hash}")
    
    # Check for changes
    nf_changed = nf_current_hash != nf_new_hash
    spec_changed = spec_current_hash != spec_new_hash
    tests_changed = tests_current_hash != tests_new_hash
    
    print(f"nf.json changed: {nf_changed}")
    print(f"spec.json changed: {spec_changed}")
    print(f"tests.json changed: {tests_changed}")
    
    # Get metadata from new files
    nf_metadata = get_file_metadata("nf_new.json")
    spec_metadata = get_file_metadata("spec_new.json")
    tests_metadata = get_file_metadata("tests_new.json")
    
    # Check if any changes detected
    has_changes = nf_changed or spec_changed or tests_changed
    
    if has_changes or force_update:
        print("Changes detected or force update requested")
        should_update = True
    else:
        print("No changes detected")
        should_update = False
    
    # Prepare metadata for output
    metadata = {
        'nf_changed': nf_changed,
        'spec_changed': spec_changed,
        'tests_changed': tests_changed,
        'nf_new_hash': nf_new_hash,
        'spec_new_hash': spec_new_hash,
        'tests_new_hash': tests_new_hash,
        'nf_created': nf_metadata['created'],
        'nf_unicode': nf_metadata['unicode'],
        'spec_created': spec_metadata['created'],
        'spec_unicode': spec_metadata['unicode'],
        'spec_cldr': spec_metadata['cldr'],
        'tests_created': tests_metadata['created'],
        'tests_unicode': tests_metadata['unicode'],
        'tests_cldr': tests_metadata['cldr'],
        'has_changes': has_changes,
        'should_update': should_update
    }
    
    return should_update, metadata


def remove_json_backups(*paths: str) -> None:
    """Remove stale .backup files from older updater runs."""
    for path in paths:
        backup = f"{path}.backup"
        if os.path.exists(backup):
            os.remove(backup)
            print(f"Removed stale backup: {backup}")


def update_files(
    nf_local: str,
    spec_local: str,
    tests_local: str,
    nf_changed: bool,
    spec_changed: bool,
    tests_changed: bool,
) -> None:
    """Replace only changed files; discard unused downloads."""
    print("Updating files...")
    remove_json_backups(nf_local, spec_local, tests_local)

    for changed, new_path, local_path in (
        (nf_changed, "nf_new.json", nf_local),
        (spec_changed, "spec_new.json", spec_local),
        (tests_changed, "tests_new.json", tests_local),
    ):
        if changed:
            shutil.move(new_path, local_path)
            print(f"Updated {local_path}")
        elif os.path.exists(new_path):
            os.remove(new_path)

    print("Files updated successfully")


def cleanup_temp_files() -> None:
    """Clean up temporary files."""
    temp_files = ["nf_new.json", "spec_new.json", "tests_new.json"]
    for file in temp_files:
        if os.path.exists(file):
            os.remove(file)


def main():
    """Main function."""
    parser = argparse.ArgumentParser(description="Check for Unicode normalization file updates")
    parser.add_argument("--force", type=str, help="Force update even if no changes detected (true/false)", default="false")
    parser.add_argument("--nf-url", default="https://raw.githubusercontent.com/adraffy/ens-normalize.js/refs/heads/main/derive/output/nf.json",
                       help="URL for nf.json file")
    parser.add_argument("--spec-url", default="https://raw.githubusercontent.com/adraffy/ens-normalize.js/refs/heads/main/derive/output/spec.json",
                       help="URL for spec.json file")
    parser.add_argument("--tests-url", default="https://raw.githubusercontent.com/adraffy/ens-normalize.js/refs/heads/main/validate/tests.json",
                       help="URL for tests.json file")
    parser.add_argument("--nf-local", default="src/static_data/nf.json",
                       help="Local path for nf.json file")
    parser.add_argument("--spec-local", default="src/static_data/spec.json",
                       help="Local path for spec.json file")
    parser.add_argument("--tests-local", default="tests/tests.json",
                       help="Local path for tests.json file")
    parser.add_argument("--update", action="store_true", help="Actually update the files")
    
    args = parser.parse_args()
    
    try:
        # Parse force argument
        force_update = args.force and args.force.lower() in ('true', '1', 'yes', 'on')
        
        should_update, metadata = check_for_updates(
            args.nf_url,
            args.spec_url,
            args.tests_url,
            args.nf_local,
            args.spec_local,
            args.tests_local,
            force_update
        )
        
        if should_update and args.update:
            update_files(
                args.nf_local,
                args.spec_local,
                args.tests_local,
                metadata["nf_changed"],
                metadata["spec_changed"],
                metadata["tests_changed"],
            )
        
        # Output results for GitHub Actions
        if 'GITHUB_ACTIONS' in os.environ:
            with open(os.environ['GITHUB_OUTPUT'], 'a') as f:
                f.write(f"nf_changed={str(metadata['nf_changed']).lower()}\n")
                f.write(f"spec_changed={str(metadata['spec_changed']).lower()}\n")
                f.write(f"tests_changed={str(metadata['tests_changed']).lower()}\n")
                f.write(f"nf_new_hash={metadata['nf_new_hash']}\n")
                f.write(f"spec_new_hash={metadata['spec_new_hash']}\n")
                f.write(f"tests_new_hash={metadata['tests_new_hash']}\n")
                f.write(f"nf_created={metadata['nf_created']}\n")
                f.write(f"nf_unicode={metadata['nf_unicode']}\n")
                f.write(f"spec_created={metadata['spec_created']}\n")
                f.write(f"spec_unicode={metadata['spec_unicode']}\n")
                f.write(f"spec_cldr={metadata['spec_cldr']}\n")
                f.write(f"tests_created={metadata['tests_created']}\n")
                f.write(f"tests_unicode={metadata['tests_unicode']}\n")
                f.write(f"tests_cldr={metadata['tests_cldr']}\n")
                f.write(f"has_changes={str(metadata['has_changes']).lower()}\n")
                f.write(f"should_update={str(metadata['should_update']).lower()}\n")
        
        # Exit with appropriate code
        # Always exit with 0 for GitHub Actions to prevent workflow failure
        # The workflow will check the should_update output to determine next steps
        sys.exit(0)
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        cleanup_temp_files()


if __name__ == "__main__":
    main()
