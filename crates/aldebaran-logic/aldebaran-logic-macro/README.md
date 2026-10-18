# Aldebaran Logic Derives

`aldebaran-logic-macro` provides derives for the assertion and classification
traits in [`aldebaran-logic`](https://docs.rs/aldebaran-logic/latest/aldebaran_logic/).
The runtime crate's prelude re-exports the derives alongside its traits and
combinators, so frontend crates can use the same dependency for both.

`Assert` assigns a field-composition rule or a delegated predicate to a struct
or each enum variant. `Choose` preserves an accepted assertion value as the
classification result and therefore requires `Clone` in addition to `Assert`.
