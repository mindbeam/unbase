
# Design Goals

* Minimize latency
  Users must have a [http://www.nngroup.com/articles/response-times-3-important-limits/](perceptably-instant) experience.
* Scalability
  The system must handle millions of nodes participants.
* Regional autonomy (geographical, planetary)
  No significant fraction of the system should be offlined due to network partition.
  Must handle significant latency between regions (minutes, hours, days), as well as relativistic effects. No, this is not a joke.
* Distributed service oriented architecture
  Advertise, execute relevant business logic from any node in the network - Via nearest-peer RMI when necessary, but optionally by replicating the action code itself.
  "Actions" should be callable synchronously, or asynchronously. (With asynchronicity being strongly preferred)
* Decouple business logic via triggers
  Authorized parties may advertise "triggers", such that business logic can be called synchronously, or (preferably) asynchronously.
* Integral push-updates
  Data replication and event notification are the same thing. Out of band event propagation is not compatible with reasonable consistency-models. (RabbitMQ is not your friend)
* Integral content-filtered subscriptions
  Allow the creation of (in-band) distributed content-matching trees.
* Integral audit trail
  Given the mechanics of replication, significant efficiency can be gained by extending the system to handle audit trails natively, rather than storing audit trails in standard tablespace.

# Functional Topology

In a nutshell: Move data inside the process, and closer to the processor.
Don't copy your working set from the database, move the relevant portion of the "database" into your process.
Per


![Example topology](./docs/Model.png)

# Consistency Model

Many debates have been had on the subject of CAP theorem, also known as Brewer's theorem:
* Consistency
* Availability
* Partition Tolerance

Conventional wisdom says *choose two* - Achieving all three is impossible.
This is undeniably correct, [IF you are using a traditional serializable/linearizable model.](https://groups.google.com/forum/#!msg/cloud-computing/nn7Sw5T0eSE/NxOTUwD_0ykJ)
That is to say, if you must conclusively know which event happened first, then you have to choose CA, CP, or AP; no ifs-ands-or-buts.

Brewer's CAP theorem dictates that the order of events among disconnected systems cannot be reliably determined.
Einstein's theory of relativity dictates that it's not in fact possible to determine this at all.
Clock-skew isn't just an irritation for satelite operators and spacecraft, it's a fundimental property of the universe.

## Explicit Causal Graph - A form of strong eventual consistency

Each object is comprised of a series of atoms, each recording an action corresponding to the creation, amendment, or deletion of the object.
Question: Is each of the fields in the edit object necessarily a CRDT?

One or more actions are initiated within the context of a transaction ID, a non-sequential generated ID which is guaranteed to be unique.

Each transaction ID in the below table is in the format of: Node ID.Transaction Counter
The first edit atom is called 

| Object ID | Trans ID  | Parent Trans IDs | Payload 
| --------- | --------- | -----------------| -------
| 123       | A.1       | NULL             | foo=1
| 123       | A.2       | A.1              | foo=3
| 123       | B.1       | A.1              | foo=9
| 123       | B.2       | A.2, B.1         | foo=9

Alternate:

| Atom ID | Object ID | Trans ID  | Parent Atom IDs | Payload 
| ------- | --------- | --------- | --------------- | -------
| 1       | 123       | A.1       | NULL            | foo=1
| 2       | 123       | A.2       | 1               | foo=3
| 3       | 123       | B.1       | 1               | foo=9
| 4       | 123       | B.2       | 1, 3            | foo=9


The Working copy of the object maintains a pre-calculated representation of it's values.
Whenever a new edit atom is replicated from another server, its parent trans ID is compared to the Trans ID(s) at the tail of the object.
If this transaction does not match, recurse through the edit atoms by their Parent Trans ID.
Once the matching Parent Trans ID is found, evaluate it, and the subsequent edit atoms in a deterministic and Convergent / Commutative fashion, and update the working copy of the object.

Occasionally, if an extremely old atom is replicated, it may become necessary to recurse quite deeply into the historical edit atoms, however; in general
The assumption in this model is that most replication will be completed relatively expediently, and that the parent transaction of the replicated atom will
usually match the Trans ID in the working copy of the object, or have to recurse relatively few times. If an atom corresponding to the parent Trans ID is not found,
the presumption is that the replicated atom arrived out of order, and must be stored until such time as the connecting atoms have appeared.

* Consider fetching connecting atoms from the replicator
* How to handle No-Earlier-Than?
* Consider using timestamp to limit recursion for non-matched atoms?
* Atoms could be LFUd out to a deep storage node



# Distance Buckets (Work in progress)

[Location location location][what_is_distance]

In a majority of database use cases, ensuring that data is consistent, and persistent is of paramount importance.
Conventional databases employ synchronous replication to ensure that writes are commited to a quorem of relevant database
nodes before considering the transaction successful. This makes sense in many cases, but the unconditional application of this approach
is too conservative to meet our ambitious design goals. Unbase seeks to achieve tunable consistency at the schema level, offering multiple
different standards of consistency to choose from.
When network disruptions occur

Unbase shall also maintain a set of buckets into which node IDs are placed Similar to the bucket system maintained by kademlia
