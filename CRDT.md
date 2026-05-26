# CRDT

**TeXas uses an operations based text-crdt for collaborative text editing.**

Simply explained, the text CRDT works by saving the text char-by-char instead of saving the text as a string, with each char having a unique ID.

The ID is made by combining the current replica ID with a counter, to make a unique ID.

Each char records the IDs of its left and right neighbors at insertion time, which helps preserve ordering and reduce interleaving anomalies during concurrent edits.

Also, instead of removing a text element all together on deletion, we set it to be a tombstone. This is so users can add and remove a char while keeping the text consistant across all replicas.