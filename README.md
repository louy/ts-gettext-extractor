# TS Gettext Extractor
![NPM Version](https://img.shields.io/npm/v/ts-gettext-extractor?style=for-the-badge)
![Crates.io Version](https://img.shields.io/crates/v/ts-gettext-extractor?style=for-the-badge)

A command line utility to generate Gettext template files (`.pot`) from Javascript/Typescript/Astro code.

Uses SWC to parse JS files.

## Supported files
`.js`, `.jsx`, `.ts`, `.tsx` and `.astro` files are scanned.

### Astro files
An `.astro` file is rewritten into TSX before parsing, so all the functions listed below work in every place JavaScript can appear in a component:

- the TypeScript frontmatter between the `---` fences,
- `{...}` expressions in the markup, both as children and in attribute values,
- `<script>` blocks, unless their tag marks them as something other than JavaScript.

`<style>` blocks and the markup itself are ignored.

```astro
---
const heading = __('Deals near you');
const count = __n('%n offer', '%n offers', offers);
---
<Layout title={__('Deals')}>
  <h1>{heading}</h1>
  <a href="/pricing" aria-label={__p('nav', 'See pricing')}>{__('See pricing')}</a>
</Layout>
```

The rewrite preserves line numbers, so references in the POT file point at the original `.astro` source.

## Usage
See help for more details
```console
$ ts-gettext-extractor --help
Generate Gettext template files from Javascript/Typescript/Astro code

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

One tagged template literal is supported, which is `__` with no variables. E.g. `` __`My string` ``

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

