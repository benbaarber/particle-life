#!/usr/bin/env python3
"""
Sync WGSL struct definitions from types.wgsl to all other .wgsl files in a directory.
"""

import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple


def extract_structs(content: str) -> Dict[str, str]:
    """
    Extract all struct definitions from WGSL content.
    Returns a dict mapping struct name to its full definition.
    """
    structs = {}

    # Pattern to match struct definitions
    # Matches: struct Name { ... }
    pattern = r"struct\s+(\w+)\s*\{[^}]*\}"

    for match in re.finditer(pattern, content, re.MULTILINE | re.DOTALL):
        struct_name = match.group(1)
        struct_def = match.group(0)
        structs[struct_name] = struct_def

    return structs


def replace_struct_in_content(
    content: str, struct_name: str, new_definition: str
) -> str:
    """
    Replace all occurrences of a struct definition in content.
    """
    # Pattern to match the specific struct definition
    pattern = rf"struct\s+{re.escape(struct_name)}\s*\{{[^}}]*\}}"

    # Replace all occurrences
    new_content = re.sub(
        pattern, new_definition, content, flags=re.MULTILINE | re.DOTALL
    )

    return new_content


def sync_wgsl_types(types_file: Path, target_dir: Path, dry_run: bool = False) -> None:
    """
    Sync struct definitions from types_file to all other .wgsl files in target_dir.

    Args:
        types_file: Path to the types.wgsl file containing canonical definitions
        target_dir: Directory containing .wgsl files to update
        dry_run: If True, only show what would be changed without modifying files
    """
    # Read the types file
    if not types_file.exists():
        print(f"Error: {types_file} not found!")
        sys.exit(1)

    with open(types_file, "r") as f:
        types_content = f.read()

    # Extract all struct definitions from types.wgsl
    structs = extract_structs(types_content)

    if not structs:
        print(f"No struct definitions found in {types_file}")
        return

    print(f"Found {len(structs)} struct(s) in {types_file}:")
    for name in structs.keys():
        print(f"  - {name}")
    print()

    # Find all .wgsl files in the target directory
    wgsl_files = [f for f in target_dir.glob("*.wgsl") if f != types_file]

    if not wgsl_files:
        print(f"No other .wgsl files found in {target_dir}")
        return

    print(f"Processing {len(wgsl_files)} file(s)...\n")

    # Process each file
    for wgsl_file in wgsl_files:
        print(f"Processing {wgsl_file.name}...")

        with open(wgsl_file, "r") as f:
            content = f.read()

        original_content = content
        changes_made = []

        # Replace each struct definition
        for struct_name, struct_def in structs.items():
            # Check if this struct exists in the file
            if re.search(rf"struct\s+{re.escape(struct_name)}\s*\{{", content):
                content = replace_struct_in_content(content, struct_name, struct_def)
                changes_made.append(struct_name)

        if changes_made:
            print(f"  ✓ Updated struct(s): {', '.join(changes_made)}")

            if dry_run:
                print(f"  [DRY RUN] Would write changes to {wgsl_file}")
            else:
                with open(wgsl_file, "w") as f:
                    f.write(content)
                print(f"  ✓ Wrote changes to {wgsl_file}")
        else:
            print(f"  - No matching structs found")

        print()


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Sync WGSL struct definitions from types.wgsl to other .wgsl files"
    )
    parser.add_argument(
        "types_file",
        type=Path,
        help="Path to types.wgsl file containing canonical struct definitions",
    )
    parser.add_argument(
        "--dir",
        type=Path,
        default=None,
        help="Directory containing .wgsl files to update (default: same as types_file)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be changed without modifying files",
    )

    args = parser.parse_args()

    types_file = args.types_file.resolve()
    target_dir = args.dir.resolve() if args.dir else types_file.parent

    print(f"Types file: {types_file}")
    print(f"Target directory: {target_dir}")
    if args.dry_run:
        print("[DRY RUN MODE - No files will be modified]")
    print()

    sync_wgsl_types(types_file, target_dir, dry_run=args.dry_run)

    print("Done!")


if __name__ == "__main__":
    main()
