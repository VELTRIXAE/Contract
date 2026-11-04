#!/usr/bin/env python3
"""Scan tracked and nonignored new files (optionally all reachable history) without printing secrets.
Heuristic checks supplement review; they cannot establish absence of credentials.
"""
import argparse
import re
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--history', action='store_true')
args = parser.parse_args()
patterns = [
    re.compile(rb'(?<![A-Z2-7])S[A-Z2-7]{55}(?![A-Z2-7])'),
    re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    re.compile(rb'(?<![A-Z0-9])(?:AKIA|ASIA)[A-Z0-9]{16}(?![A-Z0-9])'),
    re.compile(rb'gh[pousr]_[A-Za-z0-9]{36,}'),
]

def git(*argv):
    return subprocess.check_output(['git', *argv])

items = []
if args.history:
    for line in git('rev-list', '--objects', '--all').splitlines():
        oid = line.split(b' ', 1)[0].decode()
        if git('cat-file', '-t', oid).strip() == b'blob':
            items.append((oid, git('cat-file', 'blob', oid)))
else:
    from pathlib import Path
    for name in git('ls-files', '-z', '--cached', '--others', '--exclude-standard').split(b'\0'):
        if name:
            path = Path(name.decode())
            if path.is_file():
                items.append((str(path), path.read_bytes()))
failed = False
for name, data in items:
    if any(pattern.search(data) for pattern in patterns):
        print(f'FAIL potential credential in {name}; inspect privately (value suppressed)')
        failed = True
print(f'{"FAIL" if failed else "PASS"}: scanned {len(items)} {"historical blobs" if args.history else "working-tree files"}; heuristic patterns only')
raise SystemExit(1 if failed else 0)
