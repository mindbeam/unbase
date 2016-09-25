
= Unbase Consistency Model

Unbase seeks to implement a Causal consistency model, which we are calling "Infectious Knowledge"
This may be similar to "Potential Causal Consistency"

The "Infectious Knowledge" Consistency model seeks to implement the strongest consistency guarantees which are possible without coordination under presently understood physical laws, and which is feasible on modern computing hardware.

This means no shared state, no linearizability, no quorums (except for those you choose to implement as an overlay).

== Synopsis

In functional programming, it's very common to employ immutable data structures.
These data structures are elegant, and efficient. Unbase intends to extend this immutable data-structure into a distributed system which may encompass thousands, or even millions of Unbase instances, while offering strong causal consistency guarantees.


== Immutable data

In immutable data structures, when an given node is edited, a clone of that node is created with the respective change. A new parent node is created referencing the new node, as well as any other nodes which haven't changed during the operation. Continuing all the way to the "root" node. This yields a new root node which may be used to query the revised data structure.

[ Immutable DS illustration 1 here ]

TODO: Merging concurrent updates

=== Probability-based merging

The downside of immutable data structure approach is that multiple editors in the system would cause a bunch of new intermediate and root nodes to be created. This wold eventually stabilize for a given set of e=ve

== Sparse vector clock (Beacons)

TODO: Similar to interval tree clocks --
Assume you had a vector clock of unlimited width, and comparing vector clock readings is cheap.
Employ a distributed index tree as a way to locate

== Indexes

== Causal Context



* Allow continued operation during a network partition
 * Avoid CAP theorem limitations by abandoning linearizability in favor of [causal consistency](http://sns.cs.princeton.edu/projects/cops-and-eiger/)
 * Treat conflicts as inevitable, and allow them to be resolved systematically
* Destroy the distinction between client and server. They are considered identical **except** for policy, capability, and resources.
 * Access control enforcement at every stage of replication
 * Push business logic to initiators when possible, otherwise delegate to nearest capable node
* Virtualized objects, accessible from any node, complete with synchronous, asynchronous business logic enforcement
* Utilize [mesh networking](https://github.com/telehash/telehash.org/tree/master/v3) to allow ALL system participants ("clients" and "servers") to communicate directly, and around damage or network interruption


= Notes
No quorum logic shall be utilized. Provided the requisite data is available and sufficiently fresh according to its present causal context, a node, or cluster of nodes may continue functioning in the partitioned area without limitation, except as necessary to enforce durability guarantees; wherein the application logic may choose to whether to wait to reach the desired probability of durability or not.
