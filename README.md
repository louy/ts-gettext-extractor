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
- **`dgettext`** or **`__d`** — e.g. `__d('domain', 'String')`
- **`dngettext`** or **`__dn`** — e.g. `__dn('domain', '1 item', '%n items', count)`
