# TS Gettext Extractor
A command line utility to generate Gettext template files (`.pot`) from Javascript/Typescript code.

Uses SWC to parse JS files.

## Usage
See help for more details
```console
$ ts-gettext-extractor --help
Generate Gettext template files from Javascript/Typescript code

Usage: ts-gettext-extractor [OPTIONS] --output-folder <OUTPUT_FOLDER>

Options:
      --exclude [<EXCLUDE>...]
          A list of patterns to exclude [default: /.git/ /node_modules/ /__tests__/ .test. /__mocks__/ .mock. .story. .cy.]
      --path <PATH>
          The path to the file to read. Defaults to current folder
      --output-folder <OUTPUT_FOLDER>
          The folder where pot files will be written. Each domain will have its own file
      --references-relative-to <REFERENCES_RELATIVE_TO>
          Which folder the references are relative to. Defaults to the output folder
      --default-domain <DEFAULT_DOMAIN>
          The default domain to use for strings that don't have a domain specified [default: default]
  -h, --help
          Print help
```

## Supported functions

- **`gettext`** or **`__`** — e.g. `__('String')`
- **`ngettext`** or **`__n`** — e.g. `__n('1 item', '%n items', count)`
- **`pgettext`** or **`__p`** — e.g. `__p('context', 'String')`
- **`npgettext`** or **`__np`** — e.g. `__np('context', '1 item', '%n items', count)`
- **`dgettext`** or **`__d`** — e.g. `__d('domain', 'String')`
- **`dngettext`** or **`__dn`** — e.g. `__dn('domain', '1 item', '%n items', count)`
- **`dpgettext`** or **`__dp`** — e.g. `__dp('domain', 'context', 'String')`
- **`dnpgettext`** or **`__dnp`** — e.g. `__dnp('domain', 'context', '1 item', '%n items', count)`

## Metadata

This library produces a few metadata in the POT files as below.

### References
References to the code is produced in accordance with the [po file spec](https://www.gnu.org/software/gettext/manual/html_node/PO-Files.html). Each reference mentioned the source file name and line number. References are relative to the `--references-relative-to` argument (or `--output-folder`).

### Comments
Comments before or after a `gettext` function call are also extracted. This only applies to comments directly before the function call, not comments on the previous line.

For example, this WILL be extracted:
```js
const myText = /* ✅ A comment that will be extracted */ __('My text');
```

This WILL NOT be extracted:
```js
/* ❌ A comment that won't be extracted */
const myText = __('My text');
```

## Development
This library is written in rust, so you'll need rust & cargo installed. We also use `cargo-make` so make sure to install it:
```console
$ cargo install --force cargo-make
```

Look at [Makefile.toml](/Makefile.toml) for list of tools & tasks we use.
