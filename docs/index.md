---
layout: page
title: "What is Unbase?"
---

Unbase is a concept for a distributed database/application framework that is fundamentally reactive, fault tolerant, and decentralized. It seeks to address some very specific shortcomings in traditional paradigms; to create a distributed architecture that transcends device, geography, programming language, and present orthodoxy about what constitutes a "database". It seeks to blur the lines between application/database, and client/server.

We believe that for many use cases, data (or computation thereof) should _not_ be assigned to any specific storage location (as is the case in "sharded" systems) but rather it should be stored close its origin, and its consumers. Data should not be "based"<sup>[1](#footnote1)</sup> anywhere, thus the name Unbase.

*Unbase is presently under active development.*

## Summary of Design Goals:
See [Design Goals](design/goals) for more details

* Provide the strongest consistency guarantees possible with zero coordination/waiting
* Drastically reduce operational latency by focusing on data locality (planet/city/memory-bus/processor-core)
* Peer-to-peer networking to ensure continued operation during network partitions<sup>[2](#footnote2)</sup>
* A robust type system <sup>[3](#footnote3)</sup> commonly employed by RDBMS
* Tunable durability guarantees
* Reduced costs associated with hosting infrastructure, and the planning thereof.
* Common, minimalist library for client and server<sup>[4](#footnote4)</sup> applications
* Distributed content-filtered pub/sub for efficient push notifications
* Provide a facility for the registration and execution of triggers to allow for reactive, but loose couplings

## Consistency Model

Unbase seeks to implement a specific causal consistency model which we are calling "Infectious Knowledge".
See [Consistency Model](docs/CONSISTENCY_MODEL.md) for more details

<a name="footnote1">1</a>: When data storage locality is determined by an algorithm which fails to consider the points in space where the data is originated or observed, the requester must wait longer for its retrieval. See [light cones](https://en.wikipedia.org/wiki/Light_cone)<br>
<a name="footnote2">2</a>: Using the term "partition" for conversational understanding. Partitions are not actually a thing.<br>
<a name="footnote3">3</a>: entity-attribute-value, serializations stored as text, etc<br>
<a name="footnote4">4</a>: We believe there should be no difference in capability between client and server except for capability and policy.
