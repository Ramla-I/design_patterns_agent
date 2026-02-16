#!/usr/bin/env python3
"""
Consolidate individual JSON reports into a single markdown report.
"""

import json
import os
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any


def load_report(path: Path) -> Dict[str, Any]:
    """Load a JSON report file."""
    with open(path, 'r') as f:
        return json.load(f)


def get_invariant_type_display(inv_type: str) -> str:
    """Get display name for invariant type."""
    mapping = {
        'state_machine': 'State Machine',
        'linear_type': 'Linear Type',
        'ownership': 'Ownership'
    }
    return mapping.get(inv_type, inv_type)


def generate_consolidated_report(reports_dir: Path, output_path: Path):
    """Generate a consolidated markdown report from individual JSON reports."""

    # Collect all reports
    programs_with_invariants: Dict[str, Dict[str, Any]] = {}
    programs_without_invariants: List[str] = []
    failed_programs: List[str] = []

    total_invariants = 0
    total_state_machine = 0
    total_linear_type = 0
    total_ownership = 0

    # Process each JSON file
    json_files = sorted(reports_dir.glob('*.json'))

    for json_file in json_files:
        program_name = json_file.stem

        try:
            report = load_report(json_file)

            if report.get('invariants') and len(report['invariants']) > 0:
                programs_with_invariants[program_name] = report
                total_invariants += report['summary']['total_invariants']
                total_state_machine += report['summary'].get('state_machine_count', 0)
                total_linear_type += report['summary'].get('linear_type_count', 0)
                total_ownership += report['summary'].get('ownership_count', 0)
            else:
                programs_without_invariants.append(program_name)

        except (json.JSONDecodeError, KeyError) as e:
            failed_programs.append(f"{program_name} (error: {e})")

    # Generate markdown
    output_lines = []

    # Title and timestamp
    batch_name = reports_dir.name
    output_lines.append(f"# {batch_name} Invariant Analysis Report\n")
    output_lines.append(f"*Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*\n")
    output_lines.append("")

    # Summary
    output_lines.append("## Summary\n")
    output_lines.append(f"| Metric | Count |")
    output_lines.append(f"|--------|-------|")
    output_lines.append(f"| Total programs analyzed | {len(json_files)} |")
    output_lines.append(f"| Programs with invariants | {len(programs_with_invariants)} |")
    output_lines.append(f"| Programs without invariants | {len(programs_without_invariants)} |")
    output_lines.append(f"| **Total invariants found** | **{total_invariants}** |")
    output_lines.append("")

    # Invariant breakdown
    output_lines.append("### Invariant Breakdown\n")
    output_lines.append(f"| Type | Count |")
    output_lines.append(f"|------|-------|")
    output_lines.append(f"| State Machine | {total_state_machine} |")
    output_lines.append(f"| Linear Type | {total_linear_type} |")
    output_lines.append(f"| Ownership | {total_ownership} |")
    output_lines.append("")

    # Table of contents
    if programs_with_invariants:
        output_lines.append("## Quick Navigation\n")
        for program in sorted(programs_with_invariants.keys()):
            count = programs_with_invariants[program]['summary']['total_invariants']
            output_lines.append(f"- [{program}](#{program.replace('_', '-').replace(' ', '-').lower()}) ({count} invariants)")
        output_lines.append("")

    output_lines.append("---\n")

    # Programs with invariants
    if programs_with_invariants:
        output_lines.append("## Programs with Invariants\n")

        for program in sorted(programs_with_invariants.keys()):
            report = programs_with_invariants[program]

            output_lines.append(f"### {program}\n")
            output_lines.append(f"**Invariants found: {report['summary']['total_invariants']}**\n")

            # Group by type
            invariants_by_type: Dict[str, List[Dict]] = {
                'state_machine': [],
                'linear_type': [],
                'ownership': []
            }

            for inv in report.get('invariants', []):
                inv_type = inv.get('invariant_type', 'unknown')
                if inv_type in invariants_by_type:
                    invariants_by_type[inv_type].append(inv)

            for inv_type, invariants in invariants_by_type.items():
                if not invariants:
                    continue

                type_display = get_invariant_type_display(inv_type)
                output_lines.append(f"#### {type_display} Invariants\n")

                for inv in invariants:
                    output_lines.append(f"**{inv['title']}**\n")
                    output_lines.append(f"- *Location*: `{inv['location']['file_path']}:{inv['location']['line_start']}-{inv['location']['line_end']}`")
                    output_lines.append(f"- *Description*: {inv['description']}\n")

                    if inv.get('evidence', {}).get('code_snippet'):
                        output_lines.append("<details>")
                        output_lines.append("<summary>Code Evidence</summary>\n")
                        output_lines.append("```rust")
                        output_lines.append(inv['evidence']['code_snippet'])
                        output_lines.append("```")
                        if inv['evidence'].get('explanation'):
                            output_lines.append(f"\n*Explanation*: {inv['evidence']['explanation']}")
                        output_lines.append("</details>\n")

            output_lines.append("---\n")

    # Programs without invariants
    if programs_without_invariants:
        output_lines.append("## Programs without Detected Invariants\n")
        output_lines.append("The following programs were analyzed but no invariants were detected:\n")
        for program in sorted(programs_without_invariants):
            output_lines.append(f"- {program}")
        output_lines.append("")

    # Failed programs
    if failed_programs:
        output_lines.append("## Analysis Failures\n")
        output_lines.append("The following programs encountered errors during analysis:\n")
        for program in failed_programs:
            output_lines.append(f"- {program}")
        output_lines.append("")

    # Write output
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, 'w') as f:
        f.write('\n'.join(output_lines))

    print(f"Consolidated report written to: {output_path}")
    print(f"  - Programs analyzed: {len(json_files)}")
    print(f"  - Programs with invariants: {len(programs_with_invariants)}")
    print(f"  - Total invariants: {total_invariants}")


def main():
    if len(sys.argv) < 3:
        print("Usage: consolidate_reports.py <reports_dir> <output_file>")
        sys.exit(1)

    reports_dir = Path(sys.argv[1])
    output_path = Path(sys.argv[2])

    if not reports_dir.exists():
        print(f"Error: Reports directory does not exist: {reports_dir}")
        sys.exit(1)

    generate_consolidated_report(reports_dir, output_path)


if __name__ == '__main__':
    main()
