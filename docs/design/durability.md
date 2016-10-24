---
layout: page
title: "Durability"
category: design
seq: 7
disqus: 1
---
<br>

#### [Durable](https://en.oxforddictionaries.com/definition/durable){:target="define"} Adj.
  1. Able to withstand wear, pressure, or damage; hard-wearing.
  ‘porcelain enamel is strong and durable’

<br>

#### What is durability anyway?
It could be said that ceramics are durable. We make dishes, space shuttle tiles, toilets, bath tubs, teeth, tiles, and even knives from ceramics. With proper care, they tend to last a very long time indeed – But have you ever broken one of these things? Obviously, you probably have.

Of course, nothing lasts forever. Most of us learn about impermanence early on in life. In the same fashion as we each try to care for and preserve our things and ourselves, so too do we try to care for our data. That said, we care about some data a lot more than others.

<br>

#### Durability is non-boolean

Nothing is beyond loss. Computer science tends to think about durability as being a boolean state. Either we keep it, or we throw it away – but this is not correct. We seek to acknowledge a deeper truth about the nature of data. While we may have strong opinions about what we *really* want to keep, the reality is that *every datum we store has a nonzero probability of loss.* It might be due to hardware failure, hacking, physical theft, fire, network congestion, broken consistency model, common mode failure... take your pick.

<br>

#### Location assignment *decreases* durability
Most data storage systems today insist on putting every datum in a specific slot of an elaborate bento box. We tell ourselves that when there's a place for everything, and everything's in its place, that we have an orderly and durable system. We carefully manage a series of replicas of this data, trying to reduce the probability of loss. We generally keep this number of replicas fairly low, because we must coordinate to meet the guarantees offered by our consistency model.

There's a very serious downside of this approach though. When the master shard experiences a failure, it tends to be catastrophic. We risk loosing *all* of our data in that shard. We can reassign the master shard to a former slave, and have a chance of regaining normalcy, but it's dangerous to do in an automated fashion due to false positives, and you still run the risk of the entire cluster being taken out by a common failure.

If we relax the consistency model, we can store a great many copies, and ensure much higher durability! Problem solved, right? Well, no – We have to provide consistency guarantees, otherwise durability is worthless. If I break a porcelain bowl, the none of the porcelain has disappeared, but the broken shards of bowl are not going to hold my cheerios very well.

So, many copies of data is good for increasing durability. Assigning slots and coordinating among many copies is costly to manage. If we could find a way to make a bunch of copies, without having to think too hard about where everything was, that'd really be something, right?

<br>


#### Tunable consistency

If we accept that every datum has some nonzero probability of loss, why not use this to our advantage? I don't really care about throwing data away, I care about keeping the data that's important. Some data just happen to be a lot more important than others. Unbase assigns a durability target to each Memo, tunable by policy. For some data, that policy dictates that it should be highly durable, and replicated vigorously. For other data, we may not even bother with replication, or may replicate only a little bit.

Unbase seeks to take a behavioral approach, wherein peering changes or memory pressure motivate a comparison of the
calculated durability score for each memo versus the durability target for said Memo. In cases where there is substantial headroom, the Memo may be purged with an acceptably low risk of data loss over the whole system. If the durability score is below the target, then attempts shall be made to re-peer the memo, within the time constraint available.

Of course it's possible that data loss could occur when a statistically large enough set of participants are permanently shut off in a short period of time, but this is quite improbable. It's also no worse than the behavior of existing RDBMS systems under similar circumstances. Suffice to say you'll want at least *some* long-lived participants in your system. A mesh of only very short-lived processes would be unstable under any system model.

TODO: Add graphic detailing continuum of scratch data to archival data, and durability scores.
