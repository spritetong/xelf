#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re
import subprocess


def shell_run(cmd, **kwargs):
    kwargs['shell'] = True
    return subprocess.call(cmd, **kwargs)


def camel_to_snake(name):
    name = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    name = re.sub('__([A-Z])', r'_\1', name)
    name = re.sub('([a-z0-9])([A-Z])', r'\1_\2', name)
    return name.lower()


def fix_cargo_toml(path: str):
    dependencies_re = re.compile(r'^\s*\[dependencies\]\s*$')
    crate_re = re.compile(r'^(\s*)([\w-]+)(\s*=\s*\{\s*version\s*=\s*")([\d\.]+)(".*)$')

    special_versions = {}

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
            if name in special_versions:
                version = special_versions[name]
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
    shell_run('cargo upgrade')
    shell_run('cargo update')
    fix_cargo_toml('Cargo.toml')


if __name__ == '__main__':
    main()
