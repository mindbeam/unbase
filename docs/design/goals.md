---
layout: page
title: "Design Goals"
category: design
seq: 3
---

The objective of Unbase is to create a unified data/application framework, which achieves all of the following goals. Unbase could be loosely conceptualized as a sort of P2P Distributed Object Database (minus the "base")

* Minimize latency

  Give users a [perceptibly-instant](http://www.nngroup.com/articles/response-times-3-important-limits/){:target="reference"} experience whenever possible.

* Scalability

  The system must handle millions of nodes participants.

* Regional autonomy (geographical, planetary)

  No significant fraction of the system should be offlined due to network partition.
  Must handle significant latency between regions (minutes, hours, days), as well as relativistic effects.
  No, this is not a joke.

* Eliminate the distinction between client and server

  Equality for all participating nodes. Policy and capacity are the only factors which should bias decision-making.
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
