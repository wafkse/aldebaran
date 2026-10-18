# Aldebaran Text Processing

`aldebaran-text` helps compiler frontends recognize lexical forms from byte and
character sources while preserving source positions for diagnostics. Because
its lexer interface separates recognition from semantic construction, parsers
can delay conversion until the surrounding syntax is accepted.

For built-in `str` and byte-slice sources, cursor offsets and spans count bytes,
while lookahead counts source components. This lets a cursor read characters
without losing the byte positions needed for source annotations.

Cursor reads offer different consumption policies. Required reads consume
mismatches, validation preserves the position, and optional reads consume only
matches, so callers can choose whether an unsuccessful check advances input.

Branches keep cursor progress speculative until explicitly committed. However,
lexical construction has no automatic rollback of cursor or context changes,
so callers manage construction effects separately from speculative reads.

The resulting lexical values can be consumed through `aldebaran-grammar`,
allowing a parser to use the same token-stream interface with lexical output
or with producers that do not depend on this crate.
