---
layout: page
title: "The Problem"
category: design
seq: 1
disqus: 1
---

*From the spatial-overlay-as-a-service department*  

It's late on a Friday night. You're into your third hour of Guild Wars 2 and your european buddy hits you up – They want you to join their EU server. You then proceed to spend the next hour cursing the lag, and getting a lot of [rubberbanding](http://www.urbandictionary.com/define.php?term=rubberbanding){:target="define"} before you decide to call it a night.

Why does this happen? Why do you get strange behavior from the EU server when you're in Los Angeles?

The root of the problem is that for decades, we have been building computing systems on the basis of at least two major conceptual errors:

1. The belief that networks can be made reliable
2. The belief that a single arbiter of state makes a consistency-model "strong"

As our demands on computer technology have expanded, so too has the strain induced by these faulty assumptions. We as an industry have spent the past few decades trying to cope with this foundational error in conceptualization, employing endless workarounds; Yet somehow still falling short of the reliable and predictable systems we seek. These shortcomings affect systems far and wide, and in ways which span the gamut from the mundane to the profound. This generally leads to the user being disappointed in one way or another and in the undue expenditure of human capital in an attempt to combat it.

<!-- Could you boil down the main problems into bullets? I don't come away with this with a strong sense of problems XYZ exists and are manifest in most systems, "then I am going to tell you why below". -->

<br>

#### Let's face facts:

As it turns out, the universe we occupy is a little lacking in terms of cool sci-fi-physics. There's no faster-than-light travel; not for spaceships, and not even for data. Worse yet, It's not simply a matter of the galactic postal service being slow – _Existence itself_ (read: information) has an inconvenient upper limit to how fast it can travel.

<!-- I would add in something somewhere about the fact that we try to build overlays in the systems that make users think that they are physically closer than they are, so there is an appearance of information propagating faster than it can... but that it is all smoke and mirrors which eventually causes issues. Then we can move on to the next section. -->

<br>

#### Space exists...

It doesn't matter if we're talking about interstellar distances, nanometers on silicon, or anywhere in between; Information can only propagate through space so fast. Information is therefore *local*, both in terms of it's origin, and it's effects. Don't hold your breath, physicists are not optimistic about FTL transportation of information through entanglement either.

#### and simultaneity Doesn't

<!-- simultaneity might be the wrong thing to start with here? I think the bigger issue is that there is no global agreed upon ordering of events. Simultaneity is just one instance of an agreed upon order.  One where things happen at the same time.  While physicist think about simultaneity, I don't think many computer science folks do, be cause we generally pretend that things don't happen at the same time anyway.  So to me the main issue is that you can't use 'time' as an arbitrator of happens before -->

There's no such thing as simultaneity, at least not in the way most people think about it. Whether we're talking about wall clocks, atomic clocks, laser light pulses, simultaneity can only ever be a *comparative* property from the point of view of a single observer. There is no gods-eye view, no plane of simultaneity surrounding the earth. This matters, because there is no one "time", no universal reference, no [sky-hook](https://www.youtube.com/watch?v=SfvouFIVCmQ){:target="define"} which we can use to create a unified point of comparison for disparate events.

<br>

### Digging in a bit

<br>

#### Any up-to-date list
<!-- If I am the average reader, I don't know what an up to date list is or why I would care.  I think you need to frame this with 'systems use the concept of an up to date list to do XYZ'.  The problem is they can't be agreed upon throughout the whole system. -->

A vast majority of database and other systems in production today (Mysql, TCP, etc) use linearizable/serializable consistency models, in which a single arbiter manages the linearization – Otherwise it would be a free-for-all, and you'd have a graph instead of a line (IE: non-branching chain of events.)
There are various ways to phrase and think about it, but ultimately it comes down to this:

**The head of a non-branching chain can only exist at a single point in space.**

Yes, that point can move around, as in quorum / failover schemes, but one way or another everybody else has to travel to it in order to be using the same list. This is what consensus algorithms like Paxos and RAFT do: they essentially juggle or virtualize the end of the list. They give you a little bit more fault tolerance, but they're not magical teleportation devices – They come at the cost of making participants wait for a quorum of the nodes. Furthermore, consensus algorithms work sort-of-okishly in a single datacenter environment where you have a "reliable" network, but one network glitch and you're in the hurt-locker very fast. Consensus across datacenters, continents, P2P networks? forget about it.

<br>

#### Travel considered harmful - CAP Theorem

[Gilbert and Lynch](http://dl.acm.org/citation.cfm?id=564601){:target="cap"} define "consistency" as linearizability, and prove (quite factually) that interacting with an up-to-date list requires traveling in space. Being subject to alligators, backhoes, network storms, etc – traveling can at times be [quite perilous.](http://queue.acm.org/detail.cfm?id=2655736){:target="reliable"}

Unfortunately, in the course of their proofs, Gilbert and Lynch managed to [throw out the baby with the bathwater.](https://arxiv.org/abs/1509.05393){:target="kleppman"}

When we decide that a system will use a single arbiter of truth we're saying that either: We want to pretend that faster-than-light travel exists, OR that the user of the system is willing to wait for the round-trip journey to the arbiter, the success of which is not guaranteed – We might be waiting for a while.
<br>

#### Eventual Consistency – A bridge too far
<!-- I think the point to make here is that we gained intuition that data needs to be distributed.  However it was distributed mainly to increase parallelism.  We wanted to handle more requests than a single server could handle.  We also knew we didn't want to wait for a central arbiter.  So we got rid of the central arbiter and made "copies" of the data.  Now multiple servers could share the load.  the problem though, is that two users can now modify the data at two different points in space-time. Meaning that the data on the two servers is now inconsistent.  The servers will eventually exchange information and become consistent. So this is good, and is not really different in premise from what we are doing here.  So what is the big deal?  Well one thing is that they do this by making "state" consistent and having the messages / mutations being ephemeral.  Which we don't love. -->

Starting around the mid-2000s, and reeling in horror from the seemingly profound impact of the CAP theorem, database designers similarly proceeded to throw out the consistency-baby with the coordination-bathwater. Wisely seeking out [Shared-Nothing](https://en.wikipedia.org/wiki/Shared_nothing_architecture){:target="define"} systems, but then naively inducing their users to implement their own ad-hoc, poorly researched, poorly implemented consistency models as an overlay because of the missing feature – That is, consistency which is compatible with human expectations).

<br>

**We can do better.**

<br>
<!-- MM I think the point here is that sharding was again a way to distribute data around the system.  That's good.  But again they did this for parallelism, NOT for latency.  The sharding algorithms are usually rather arbitrary in terms of data locality.  There is just some coordinator or pre-agreed upon algorithm that places data at arbitrary locations in the system.  Good for parallelism, bad for understanding the no-FTL issue of the universe. -->

<!--- WIP section. Overly wordy, underdeveloped

#### Sharding is just another word for patience

A priori sharding works great for department stores. Looking for that fresh pair of cat-themed socks? You can go look at the directory, and generally find the right area without too much fuss. Now, imagine the department store was *really* large and spread out. You're in kitchen appliances now, and the sock department is 1,000 miles away. You're going to be walking for a while. How committed are you to getting those cat socks again?

Instead of putting them all in the one place, what if store management chose to distribute the goods around the store? They have 50 pairs of cat socks. Why not sprinkle them around randomly, mixing them on the shelf with food processors, lingere, perfume, and biscotti? Chances are there will be a pair which is much closer to where you are! The hard part is that now you have one pair of socks each in 50 different places around the store. That store directory isn't going to work anymore. You're going to have to get creative.

Your new store directory could utilize consistent hashing, and the stock person would simply put each pair of socks in each of the deterministically designated places in the store.
That would work, but you might still have to walk 20 miles or more to get them! Admittedly this is an improvement over 1,000 miles, but still a pretty rough walk for some cat socks.

What if instead of using deterministic slot assignment, we simply dropped the the socks off close to wherever they were arrived, and let people move them around the store as they saw fit? Chances are the store employees putting them away will at least consider buying a pair (how could they not?) Later, as shoppers browse, they might pick up a pair. Some will change their minds and put the article down. Chances are, enough shoppers will shuffle them around the store over time that they will become optimally placed for some of the other cat-fanciers. In our case, the socks are infinitely copyable immutable data, and we never actually leave the store.

<br>

-->

#### End-to-end

The simple truth is that system implementers _must_ reason about their data from end to end; not just inside the walled-garden of their "database" consistency model. We assert that those who fail to reason about this plethora of consistency-models which span the gap are planning for failure. Most systems today implement no fewer than five different consistency models. Most implementers only tend to think about the first, and scratch their heads when weird stuff happens (or politely inform you that "you're holding it wrong")

Example of the consistency models that you might not be thinking about:

* Relational database (linearizable/serializable)
* TCP Connection to the RDBMS (linearizable)
* RDBMS client / In-process pool of TCP connections (ad hoc)
* Caching system for your service (ad-hoc, possibly wallclock-based)
* Connection between the user's client app and your service, including TCP/haproxy load balancer (ad-hoc, eventual consistency)
* Caching in your end client application (ad-hoc, possibly wallclock)

As a developer, getting your head around just one consistency model is often hard enough, but several? Forget about it. Is it any wonder why our applications are so flakey these days? Ever used Slack while getting on an elevator? Ever pressed the checkout button on a shopping cart page twice? Ever had to deal with a network outage at your datacenter? How big is your global ops team, and how much of your engineering budget to you spend on cache invalidation? Modern software is overrun with examples of multiple ad-hoc, poorly conceptualized consistency models causing problems in everyday life.

(Some wonderful folks are working on applying CRDTs to try to solve these problems. While this is a good start, we do not believe that approach goes quite far enough.)

<br>

#### When in Rome

The physical reality around us doesn't have centralized arbiters of truth, It's decentralized.
When I set down my glass on the table, it doesn't have to coordinate with a datacenter in Ashburn, VA to avoid spontaneously jumping to the opposite side of the table. It has local, *causal*, **coordination free** consistency. So too, should our systems. This consistency model is totally aligned with our perspective as humans, because it's the same consistency model we were born into.

Lets learn from natural systems and relax. Nature doesn't do a priori resource planning, and neither should we.

<img src="/unbase_globe_96x96.png" style="height:48px; float: right">
