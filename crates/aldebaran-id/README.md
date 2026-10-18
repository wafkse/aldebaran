# Aldebaran Identifiers

`aldebaran-id` provides nonzero integer identities, optional compile-time type
tags, and collections keyed by IDs. An ID does not establish that its value is
unique or that the referenced object remains in storage. Resolve storage-issued
IDs through their owning storage.

`ident` supplies the identity contract and type tags, while `zeroable` preserves
nonzero values across generic integer types. `array` groups IDs by position,
and `map` and `set` associate stored entries with ID equality. Map slice views
pair keys and values by position, but those positions may change on removal.
