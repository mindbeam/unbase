---
layout: page
title: "Consistency Model"
category: design
seq: 2
disqus: 1
---

#### What is a consistency model?

A Consistency Model is a contract between the system and the user of the system which specifies a set of invariants to which the system must adhere. This allows users and developers alike to set their expectations and reason about the behavior of the system.

**We are calling the Unbase consistency model "Infectious Knowledge"**

---  

### Potential Causality
*Infectious Knowledge* is similar to Potential Causality, insofar as it intends to guarantee that all potential causations are accounted for when projecting state for a given observer. The main difference is that under the _Infectious Knowledge_ model, the system is willfully ignorant of some concurrent causal threads which may be inside of the receiving light cone. These causal threads are assimilated on an as-needed basis, rather than an immediate basis.

<br>

### Why Causality??

<br>

#### 1. Because we're Lazy
We *really* don't like coordinating with other parties. Not just because it's inconvenient, it's also dangerous! Even if we accepted the latency of a round-trip around the globe ([or further](https://en.wikipedia.org/wiki/Interplanetary_Transport_System){:target="spacex"},) we could run into some serious problems while traveling!
* Power Outages
* Overloaded routers
* Packet floods
* Backhoes in Nebraska
* FSB cutting submarine cables
* Byzantine Generals
* [Byzantine Dictatorships](http://www.dailydot.com/layer8/turkey-censorship-real-life/){:target="define"}
(take your pick)

There's no such thing as a perfect network, and even if there was, we'd *still* have to wait around for it. No thanks.
This means no shared state, no linearizability, no quorums (except for those you choose to implement as an overlay).

#### 2. Because we don't like weird stuff

I set my glass of bourbon down on the table, and it tends to stay where I put it. Notwithstanding some sneaky beverage thief, it tends not to jump around the table when I blink. I like it when I stir the cream *into* (and not out of) my coffee, and when the toothpaste comes out of the tube.

All of these are enforced by a decentralized causal system called the universe – *Nature's consistency model*
Most of this silly business about linearizability/serializability is really just trying to stop weird stuff from happening. We may need linearizability for some stuff, like contiguous sequences, but we don't need it to model causality. We can get causality just about for free!
