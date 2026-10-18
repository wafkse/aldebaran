# Aldebaran Interning

`aldebaran-interner` defines borrowed and owned contracts for assigning compact
identities to values. The distinction lets implementations retain source borrows
or own their stored representation while exposing lookup through keys.
Successful storage provides a key that resolves to the stored value in that
interner, but the traits do not require equal inputs to share a key. Type-tagged
keys distinguish value types without proving which interner issued them.

When interning Aldebaran source values, the optional `source-storage` feature
provides implementations of those contracts. Their identities are scoped to the
storage that assigns them, so lookup must use the corresponding storage.
