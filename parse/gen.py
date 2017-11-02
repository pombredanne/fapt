#!/usr/bin/env python3
import collections
# fields apt likes, and we like too; we're going to parse them
import os
import re

HANDLED_FIELDS = {
    # core
    'Package',
    'Source',
    'Version',

    # mapped into proper types
    'Priority',
    'Architecture',
    'Format',

    # parsed into Binaries
    'Binary',
    'Package-List',

    # parsed into Files
    'Files',

    # typo of Original-Maintainer, upstart in xenial
    'Orig-Maintainer',

    # parsed build-deps
    'Build-Conflicts',
    'Build-Conflicts-Arch',
    'Build-Conflicts-Indep',
    'Build-Depends',
    'Build-Depends-Arch',
    'Build-Depends-Indep',

    # folded into Files
    'Checksums-Md5',
    'Checksums-Sha1',
    'Checksums-Sha256',
    'Checksums-Sha512',
}

# What a mess.
for vcs in [
    'Arch',
    'Browse',
    'Browser',
    'Bzr',
    'Cvs',
    'Darcs',
    'Git',
    'Hg',
    'Mtn',
    'Svn',
]:
    HANDLED_FIELDS.add('Vcs-' + vcs)
    HANDLED_FIELDS.add('Orig-Vcs-' + vcs)
    HANDLED_FIELDS.add('Original-Vcs-' + vcs)
    HANDLED_FIELDS.add('Debian-Vcs-' + vcs)
    HANDLED_FIELDS.add('Upstream-Vcs-' + vcs)
    HANDLED_FIELDS.add('Vcs-Upstream-' + vcs)

# finding new fields:
# ../raw/build/apt-dump raw-sources | cargo run --release | capnp decode ../apt.capnp Source --short | sed -n 's/.*unparsed = (//p' | sed 's/", /"\n/g' | cut -d= -f 1 | sort | uniq -c | sort -n

KNOWN_FIELDS = [
    # definitely just normal strings, don't need parsing
    'Directory',
    'Homepage',
    'Standards-Version',
    'Section',

    # should we parse out humans? Probably yes. It's full of \xescapes. Definitely yes.
    'Maintainer',
    'Original-Maintainer',
    'Uploaders',

    # should enum up Testsuite, and parse package list out of Triggers
    # https://anonscm.debian.org/git/lintian/lintian.git/tree/checks/testsuite.pm
    'Testsuite',
    'Testsuite-Triggers',
    'Testsuite-Restrictions',

    # booleans?
    'Autobuild',
    'Dm-Upload-Allowed',

    # Related fields to be simplified
    'Description',
    'Description-md5',

    # Other things apt reads
    'Breaks',
    'Bugs',
    'Built-For-Profiles',
    'Built-Using',
    'Class',
    'Conffiles',
    'Config-Version',
    'Conflicts',
    'Depends',
    'Enhances',
    'Essential',
    'Filename',
    'Files',
    'Important',
    'Installed-Size',
    'Installer-Menu-Item',
    'Kernel-Version',
    'MD5sum',
    'MSDOS-Filename',
    'Multi-Arch',
    'Optional',
    'Origin',
    'Package-Revision',
    'Package-Type',
    'Pre-Depends',
    'Provides',
    'Recommended',
    'Recommends',
    'Replaces',
    'Revision',
    'SHA1',
    'SHA256',
    'SHA512',
    'Size',
    'Source',
    'Status',
    'Subarchitecture',
    'Suggests',
    'Tag',
    'Task',
    'Triggers-Awaited',
    'Triggers-Pending',

    # Fields that have been seen in the wild, but which apt ignores.
    'Extra-Source-Only',

    'Build-Indep-Architecture',

    'Dgit',

    'Go-Import-Path',
    'Python-Version',
    'Python3-Version',
    'Ruby-Versions',

    'Comment',
]

ALIASES = {
    'Package_Revision': 'Package-Revision',
    'Orig-Maintainer': 'Original-Maintainer'
}

HANDLED_FIELDS.update(ALIASES.keys())


def to_snake(s: str) -> str:
    return re.sub(r'(?!^)[_-]([a-zA-Z])', lambda m: m.group(1).upper(), s.lower())


def to_rust(s: str) -> str:
    return re.sub(r'[_-]', '_', s.lower())


def main():
    text_fields = []
    for field in KNOWN_FIELDS:
        if field not in HANDLED_FIELDS:
            text_fields.append(field)

    max_len = max(len(to_snake(field)) for field in text_fields)
    capnp_format_string = ('    {: <' + str(max_len) + '} @{} :Text;\n')
    rust_format_string = '        "{}" => blank_to_null(val, |x| builder.set_{}(x)),\n'

    with open('../apt.capnp~', 'w') as tmp:
        with open('../apt.capnp') as orig:
            for line in orig:
                tmp.write(line)
                if '## generated by gen.py' == line.strip():
                    break

        tmp.write("""
struct UnparsedSource {
""")
        for i, field in enumerate(text_fields):
            tmp.write(capnp_format_string.format(to_snake(field), i))

        tmp.write("}\n")

    os.rename('../apt.capnp~', '../apt.capnp')

    with open('src/fields.rs', 'w') as f:
        f.write("""// GENERATED by gen.py; do not edit
#![cfg_attr(rustfmt, rustfmt_skip)]

use apt_capnp::unparsed_source;
use errors::*;
use blank_to_null;

pub const HANDLED_FIELDS: [&'static str; """ + str(len(HANDLED_FIELDS)) + """] = [
""")
        for field in sorted(HANDLED_FIELDS):
            f.write('    "{}",\n'.format(field))

        f.write("""];

pub fn set_field(key: &str, val: &str, builder: &mut unparsed_source::Builder) -> Result<()> {
    match key {
""")
        for orig in sorted(text_fields):
            f.write(rust_format_string.format(orig, to_rust(orig)))

        f.write("\n        // Typos\n")
        for key, val in ALIASES.items():
            f.write(rust_format_string.format(key, to_rust(val)))

        f.write("""
        other => bail!("unrecognised field: {}", other), 
    }

    Ok(())
}
""")


if __name__ == '__main__':
    main()
