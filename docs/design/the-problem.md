---
layout: page
title: "The Problem"
category: design
seq: 1
---

*From the spatial-overlay-as-a-service department*  

It's late on a Friday night. You're into your third hour of Guild Wars 2 and your european buddy hits you up – They want you to join their EU server. You then proceed to spend the next hour cursing the lag, and getting a lot of [rubberbanding](http://www.urbandictionary.com/define.php?term=rubberbanding){:target="define"} before you decide to call it a night.

Why does this happen? Why do you get strange behavior from the EU server when you're in Los Angeles?

<br>

#### Let's face facts:
As it turns out, the universe we occupy is a little lacking in terms of cool sci-fi-physics. There's no faster-than-light travel; not for spaceships, and not even for data. Worse yet, It's not simply a matter of the galactic postal service being slow – _Existence itself_ (read "information") has an excruciatingly inconvenient upper limit to how fast it can travel.

<br>

#### Space exists

It doesn't matter if we're talking about interstellar distances, nanometers on silicon, or anywhere in between; Information can only propagate through space so fast. Information is therefore *local*, both in terms of it's origin, and it's effects. Don't hold your breath, physicists are not optimistic about FTL transportation of information through entanglement either.

#### ...and simultaneity Doesn't

There's no such thing as simultaneity, at least not in the way most people think about it. Whether we're talking about wall clocks, atomic clocks, laser light pulses, simultaneity can only ever be a *comparative* property from the point of view of a single observer. There is no gods-eye view, no plane of simultaneity surrounding the earth.

<br>

### Digging in a bit

<br>

#### Why is coordination a problem?

When we decide that a system will use a single arbiter of truth (via linearizability usually) we're saying that either: We want to pretend that faster-than-light travel exists, OR that the user of the system is willing to wait for the round-trip journey to the arbiter.

The sales pitch: "Want high availability? No problem! Paxos and RAFT have you covered!""
Except no. Any system capable of consensus is by definition a linearizable system, and thus have major limitations, and many undesirable failure modes.

Put concisely, an up to date list can only exist at a single point in space. Sure, that point can move around, but everybody else has to travel to it. This is what consensus algorithms like Paxos and RAFT do: they essentially juggle the end of the list to make sure no one node can (in theory) take you offline, at the cost of making everybody else wait for the latency of a quorum of the nodes. Consensus algorithms work sort of ok in a single dataceter environment where you have a "reliable" network, but have a network glitch and you're in the hurt-locker very fast. Uncool. Consensus across dataceters? P2P networks? forget about it.

<br>

#### Eventual consistency to the rescue?

Ok, so coordination is bad right? [Gilbert and Lynch](http://dl.acm.org/citation.cfm?id=564601){:target="cap"} define "consistency" as linearizability, and prove (quite factually) that interacting with an up-to-date list requires traveling in space. Being subject to alligators, backhoes, network storms, etc; traveling can at times be [quite perilous.](http://queue.acm.org/detail.cfm?id=2655736){:target="reliable"} Unfortunately, in the course of their proofs, Gilbert and Lynch managed to [throw out the baby with the bathwater.](https://arxiv.org/abs/1509.05393){:target="kleppman"}

Reeling in horror from the seemingly profound impact of the CAP theorems, so too did database designers proceed to throw the baby out with the bathwater for the next decade after that. Wisely seeking out Shared-Nothing systems, but then proceeding to throw strong consistency models out the window.

#### Sharding is just another word for patience

A priori sharding is a no-no.

#### Distributed ≠ Decentralized



#### When in Rome

The physical reality around us doesn't have centralized arbiters of truth, It's decentralized.
When I set down my glass on the table, it doesn't have to coordinate with a datacenter in Ashburn to avoid spontaneously jumping to the opposite side of the table. It has local, *causal*, **coordination free** consistency. So too, should our systems. This consistency model is totally consistent with our perspective as humans, because it's the same consistency model we were born into.
