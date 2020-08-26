# sh-inline

This was forked from https://github.com/tcr/commandspec - there
were various unreviewed PRs (e.g. https://github.com/tcr/commandspec/pull/10)
and this version also has various other changes:

- Focuses just on the macros for execution and doesn't try to
  include other things like killing subprocesses.
- Documentation
- Uses "bash strict mode" http://redsymbol.net/articles/unofficial-bash-strict-mode/
- Supports `Path` objects directly (including quoting non-UTF8 values)
- Removes non-bash macros (and Windows support) - for now

## License

MIT or Apache-2.0, at your option.
