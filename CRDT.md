# CRDT

**TeXas uses an operations based text-crdt for collaborative text editing.**

Simply explained, the text CRDT works by saving the text char-by-char instead of saving the text as a string, with each char having a unique ID.
The ID is made by combining the current replica ID with a counter, to make a unique ID.
Each char records the IDs of its left and right neighbors at insertion time, which helps preserve ordering and reduce interleaving anomalies during concurrent edits.
Also, instead of removing a text element all together on deletion, we set it to be a tombstone. This is so users can add and remove a char while keeping the text consistent across all replicas.

## Data model

The document is stored as a sequence of text elements. Each element contains a character, a unique element ID, insertion-time neighbor IDs and a deleted flag.

## Operations

The CRDT has two text operations:

- `Insert`: creates a new element with an element ID, value and left/right neighbor IDs.
- `Delete`: marks an existing element as deleted.

Both operation IDs and element IDs are made from the replica ID and a local counter.

## Ordering

When a character is inserted, the CRDT records the IDs of the visible elements to its left and right at insertion time. These IDs are used later to rebuild a deterministic element order, even if operations arrive in different orders on different replicas.

## Reading the text

A value function reads the CRDT as a normal string. It first uses the deterministic element order, then skips elements marked as deleted and finally collects the remaining character values.

## Out-of-order operations

If an insert arrives before its left neighbor, TeXas buffers it until the missing neighbor arrives. If a delete arrives before the inserted element, the delete is remembered and applied when the element is later materialized.

## Sync

Peers broadcast text operations as they edit. They can also request missing operations from another peer using sync request and sync response messages.

## Limitations

The current implementation keeps tombstones forever and does not yet include garbage collection, access control, undo/redo, or a durable shared operation log.
