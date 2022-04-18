#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re
import subprocess


def fix_cargo_toml(path: str):
    ignored = []
    pinned = {}

    dependencies_re = re.compile(r'^\s*\[dependencies\]\s*$')
    crate_re = re.compile(r'^(\s*)([\w-]+)(\s*=\s*(?:\{\s*version\s*=\s*)?")([\d\.]+)(".*)$')

    with open(path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    changed = False
    in_deps = False
    for i, line in enumerate(lines):
        if not in_deps:
            if not dependencies_re.match(line):
                continue
            in_deps = True
        
        found = crate_re.findall(line)
        if found:
            name = found[0][1]
            version = found[0][3]
            if name in ignored:
                continue
            if name in pinned:
                version = pinned[name]
            elif not version.startswith('^'):
                version = '^' + '.'.join(version.split('.')[:2])
            if version != found[0][3]:
                lines[i] = '{}{}{}{}{}\n'.format(found[0][0], name, found[0][2], version, found[0][4])
                changed = True

    if changed:
       with open(path, 'w', encoding='utf-8') as f:
           f.writelines(lines)


def main():
    os.chdir(os.path.dirname(__file__))
    subprocess.call('cargo upgrade', shell=True)
    subprocess.call('cargo update', shell=True)
    fix_cargo_toml('Cargo.toml')


if __name__ == '__main__':
    main()
