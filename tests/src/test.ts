const __d = (...args: any[]) => {};

export default /* 
Another comment to be extracted. However, this time it's a very long comment that can span multiple lines and might be broken into multiple lines in the output `.po` file.
*/ __d('typescript', 'Hello there!');

__(`Test backticks`);
__d('test', `Test backticks with __d`);

__("'Single quote'");
__(
  '"Double quotes at exactly 76th character of this string... do you like that"'
);
__`And a
new line!`;


__dp("duplicates", "Fancy context", "Duplicate string");
__dp("duplicates", "Fancy context", "Duplicate string");

__(`A multi-line string
spanning two lines`);

__p('A context\nwith a new line', 'A string with a multi-line context');

__n(`%d line
break`, `%d line
breaks`, 2);

__('A literal \\n is not a line break');
