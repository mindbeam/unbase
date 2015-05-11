
# Design Goals

The objective of Unbase is to create a unified data/application framework, which achieves all of the following goals.
Unbase could be loosely conceptualized as a sort of P2P Distributed Object Database ( minus the "base" )

* Minimize latency

  Users must have a [perceptably-instant](http://www.nngroup.com/articles/response-times-3-important-limits/) experience.

* Scalability

  The system must handle millions of nodes participants.

* Regional autonomy (geographical, planetary)

  No significant fraction of the system should be offlined due to network partition.
  Must handle significant latency between regions (minutes, hours, days), as well as relativistic effects.
  No, this is not a joke.

* Eliminate the destinction between client and server

  Equality for all participating nodes. Policy and capacity are the only factors which should bias decisionmaking.
  The client/server model is flawed. Let browsers and other nodes formerly conceptualized as "clients" talk to each other, and even implement core business logic.
  
  Verify outcomes as part of the replication process, to ensure that they are authorized.

* Distributed service oriented architecture

  Advertise and execute relevant business logic from any node in the network - By way of delegation to the nearest able peer when necessary, but optionally by replicating the action code itself to the calling node.
  "Actions" should be callable synchronously, or asynchronously. (With asynchronicity being strongly preferred)
  
* Decouple business logic via triggers

  Authorized parties may advertise actions as "triggers", such that business logic can be initiated automatically on the basis of record edits.
  These triggers may be synchronous, or asynchronous, with the latter being strongly preferred for most use cases.

* Integral push-updates

  Data replication and event notification are the same thing. Out-of-band event propagation is nigh impossible to coordinate with in-band consistency models. (RabbitMQ is not your friend)
    
* Integral content-filtered subscriptions

  Create and manage distributed content-matching trees, such that interested parties may subscribe to record edits/additions/removals on the basis of a query expression.

* Integral audit trail

  Given the mechanics of distributed data replication, significant efficiency can be gained by extending the system to handle audit trails natively, rather than storing audit trails in standard tablespace.

# Functional Topology

  In a nutshell: Move data inside the process, and closer to the processor. Don't copy your working set from the database, move the relevant portion of the "database" into your process.

![Example topology](./docs/Model.png)

# Consistency Model

Many debates have been had on the subject of CAP theorem, also known as Brewer's theorem:
* Consistency
* Availability
* Partition Tolerance

Conventional wisdom says *choose two* - Achieving all three is impossible.
This is undeniably correct, [IF you are using a traditional serializable consistency model.](https://aphyr.com/posts/313-strong-consistency-models)
That is to say, if you must conclusively know which event happened first/second/third, then you have to choose either CA, CP, or AP; no ifs-ands-or-buts.

Brewer's CAP theorem is effectively saying that the order of events among disconnected systems cannot be deterministic, unless you wait until it's reconnected.
Taking things a few steps further, Einstein's theory of relativity suggests that it's not in fact possible to determine this with any certainty at all.
Why? Clock-skew isn't just an irritation for satelite operators and spacecraft, it's a fundimental property of the universe we inhabit.

We puny humans have a common sense understanding of event sequence, but this is only within certain frames of reference.
What is actually happening in reality is a life-sized causal graph, where all happenings are dictated via causal couplings, rather than a single timeline.
Rather than trying to place all events on a single logical timeline, we can instead approximate and store [explicit referential causality data](http://sns.cs.princeton.edu/projects/cops-and-eiger/) in the metadata.
So, using causality as the basis of our consistency model is not only achievable, but it is also more compatible with the basic laws of physics.

To use an extreme example for illustrative purposes:

If two parties are one light-year apart, they may act concurrently; however, it is not possible for them to transmit information faster than the speed of light.
Therefore, the time required for the actions of one party to influence the other *in any way* is one year (at a minimum).
As a result a universal sequence of actions (linearizability) is neither possible, nor relevant to their respective experiences.
The only relevant factor is causal influence of having received it.

We tend to think of earth as being in a single temporal frame of reference, and give or take a couple hundred miliseconds, it's not an unreasonable simplification.
*However* When dealing with high frequency roundtripping/decisionmaking, this matters a lot more.

We can also model a network partition simply as higher-than-normal latency.
For our purposes, there is *no meaningful difference* between fifteen minute outage due to a backhoe, and the light-travel-time to communicate with a node on Mars.
Much like Mars explorers carrying their own food and water, so too, must our system go about it's business regionally.

It is for this reason that we select a [Causal+](http://www-bcf.usc.edu/~wyattllo/papers/cops-poster-istccc.pdf) consistency model.

## Explicit Causal Graph - A form of strong eventual consistency

Each object is comprised of a series of atoms, each recording an action corresponding to the creation, amendment, or deletion of the object.
Question: Is each of the fields in the edit object necessarily a CRDT?

One or more actions are initiated within the context of a transaction ID, a non-sequential generated ID which is guaranteed to be unique.

Note: bold indicates peering reference

Each transaction ID in the below table is in the format of: Node ID.Transaction Counter

Transaction Log

| Trans ID | Parent Trans IDs | Action | Initiating User | Endorsing User | Signature
| -------- | ---------------- | ------ | --------------- | -------------- | ---------    
| A.T1     |                  | Begin  | 1               | 2              | XXXXXXXX
| A.T1     |                  | Commit | 1               | 2              | XXXXXXXX
| A.T2     | A.T1             | Begin  | 1               |                | YYYYYYYY
| A.T2     | A.T1             | Commit | 1               |                | YYYYYYYY


Edit Log

| Edit ID | Object ID | Trans ID | *Parent Edit IDs* | Payload
| ------- | --------- | -------- | ----------------- | -------       
| A.E1    | 123       | A.T1     | NULL              | foo=1
| A.E2    | 123       | A.T2     | A.E1              | foo=3
| A.E3    | 123       | B.T1     | A.E1              | foo=9
| A.E4    | 123       | B.T2     | A.E2, B.E1        | foo=9


Peering

Every Object must publish its peering status whenever moved, cloned, or decloned. Object peering is not specific to any edit. Any node which believes it has the HEAD edit for a given object should be included in this peering. Non-HEAD edits should not participate in object peering.
Object references to other objects themselves MUST participate in the peering for the referenced object.

Every Edit must publish its peering status whenever moved, cloned, or decloned.

| Object ID | Edit ID | Node      |  
| --------- | ------- | --------- |
| A.E1      | NULL    | A,B,C     |
| A.E1      | A.E2    | A,B,C     |



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
