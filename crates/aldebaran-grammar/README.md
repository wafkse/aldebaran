# Aldebaran Grammar

`aldebaran-grammar` defines a pull protocol between token producers and parsers.
A read yields a token, a source error, or `None` when no token is available at
that moment. `None` does not promise that later reads will also be empty.
Parsers can therefore choose their own grammar policy while preserving source
failures.

`aldebaran-text` uses this protocol for lexical output, but the stream contract
also allows other token producers to participate without depending on text
processing.
