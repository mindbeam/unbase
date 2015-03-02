
# Design elements

* Distributed (Top-down) B-Tree
 * Is deletion desirable? If so, is proper B-Tree pruning desirable?
* Replicated nodes synchronize changes to each other
* Employ MVCC to eliminate locking
 * Configurable MVCC history persistence for integral audit trail ( with edit metadata )
 * Configurable consistency per object, allow some transactions to complete with reduced confirmation

![Example topology](./docs/Model.png)


 
# Distance Buckets

[Location location location][what_is_distance]

In a majority of database use cases, ensuring that data is consistent, and persistent is of paramount importance.
Conventional databases employ synchronous replication to ensure that writes are commited to a quorem of relevant database
nodes before considering the transaction successful. This makes sense in many cases, but the unconditional application of this approach
is too conservative to meet our ambitious design goals. Unbase seeks to achieve tunable consistency at the schema level, offering multiple
different standards of consistency to choose from.
When network disruptions occur

Unbase shall also maintain a set of buckets into which node IDs are placed Similar to the bucket system maintained by kademlia
