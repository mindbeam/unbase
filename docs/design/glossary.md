---
layout: page
title: "Glossary"
category: design
seq: 5
---

<br>

#### <a name="durability-score">Durability Score</a>

A calculated probability that a given [Memo](#memo) will not be lost. Each [Slab] calculates and manages the Durability Score for each resident [Memo](#memo) This score is compared with the Memo's Durability Target, and certain behaviors are triggered depending on whether the score is above, or below the target, and by how much.

<br>

#### <a>Durability Target</a>

The desired minimum [Durability Score](#durability-score) for a given [Memo](#memo)
This value may be set by policy for a given subject, or defaulted by system policy.

<br>

#### <a name="materialzed-memo">Materialized Memo</a>

A Materialized is a kind of [Memo](#memo) which directly contains a fully materialized projection of state for the relevant [Object](#object) given a specific set of precursors. It also contains a [Causal Reference](#causal-reference) to each of these immediate precursors, but does not need to traverse them in order to perform a state projection.

Materialized Memos are a necessary optimization in order to make the system work efficiently. They may be invalidated if non-descendent Memos are observed. Their [Durability Target](#durability-target) is conditional. The default policy is that they should be Highly Durable, unless a non-descendent, non-referenced Memo appears.

Materialized Memos may effectively be invalidated, but as with all other Memos, they are never intentionally expunged. They merely fade away when unneeded.

<br>

#### <a name="memo">Memo</a>

A single immutable message.

Properties of a Memo:

* May specify an [Object](#object)
* Identified by the hash of their contents and precursors
* Originated on a single [Slab](#slab)
* Able to be replicated across other Slabs
* May have a payload
* May reference other related memos.
* Has a [Peering](#peering) with its Replicas and References

<br>

#### <a name="object">Object</a>

An Object is conceptually similar to a record in an RDBMS, however it does not really exist, or maintain state. It is simply an enumeration, and exists only as a coalescence of it's projected [Memos](#memo). The originating Memo, and any [Materialized Memos](#materialzed-memo) for each Object specifies its [Topic](#topic).

<br>

#### <a name="peering">Peering</a>

A Peering is a gossip network across the system between all copies of one or two different Memos.
Each Memo participating in a peering may only be aware of a small subset of all copies of these Memo, but in aggregate, this is sufficient to create a fully-connected mesh.

There are two main types of Peering:
1. Replica - Peering between a Memo and other copies of itself. Necessary for [Durability Scoring](#durability-score) and Durability Management.
2. Relationship - Peering between a Memo and other referenced memos. Necessary for reference traversal

<br>

#### <a name="relationship">Relationship</a>

Each Memo may point to one or more other Memos which are related. Similar to a foreign-key in an RDBMS.
Relationships for a given Memo are dictated by the Model of its [Topic](#topic).

<br>

#### <a name="slab">Slab</a>

An agent which possesses storage and computational facilities, and implements the core behaviors which are essential to the operation of the system. A slab is similar to what most might think of as a "node", except that you may choose to have multiple slabs corresponding to different threads or processes.  

Properties of a Slab:  

* Stores [Memos](#memo) for the duration of its lifetime
* Has specific storage/network/compute quotas (MB/GB/TB+)
* Track peering data with memo replicas (gossip, not all replicas)
* Reports it's approximate expected lifetime (seconds~months+) to peers
* Calculates a [Durability Score](#durability-score) for all contained Memos
* Push replicas of memos below durability-threshold to peers
* Send heartbeat Memos to peer slabs
* Recalculate replication factor reduction for stale peers
* Accepts memo replicas from peers when below quotas
* Evicts least recently used Memos when network/storage quotas are exceeded
* Update peer slabs for memos leaving/entering

<br>

#### <a name="topic">Topic</a>

A Topic is similar to table in an RDBMS. Each [Object](#object) specifies a Topic.
