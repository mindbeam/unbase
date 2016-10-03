---
layout: page
title: "The Problem"
category: design
seq: 1
---

#### Everything sucks (No ansibles)

Ok, so you're into your third hour of Guild Wars 2, and your european buddy hits you up – They want you to join their EU server.
"No way, I get mad rubberbanding when I join the EU servers, yo" you tell them.

So why does this happen? Why do you get [rubberbanding](http://www.urbandictionary.com/define.php?term=rubberbanding){:target="define"} on an EU server when you're in Los Angeles, and your ping is high?

<br>

**Because the universe sucks.**

The universe we occupy is super lame. There's no faster-than-light travel; not for spaceships, and not even for data.
It's kind of weird and lonely when you think about it. We're each on our own little islands, isolated in our respective existences by dozens of nanoseconds, and that's assuming we're in the same room! Heck, your consciousness is separated from even your own feet by at least 4 or 5 nanoseconds.

When we're in a virtual world like Guild Wars 2, we still have to contend with the fact that it's an overlay to the physical universe. No matter how hard we try, we can never fully participate in virtual worlds that are misaligned in space from our physical world. (It's a Pretty sweet argument for AR > VR though)

The good folks at NCSOFT know this, which is why they tell you where the servers are – so you can choose one which will give you a better gaming experience.

So, why then, when we're talking about business systems, does the industry cling to the illusion that virtual systems need not map to to physical (spatial) reality? When we declare that a certain system is the arbiter of truth (linearizability) we're saying that either: We want to pretend that faster-than-light travel exists, OR that the consumer of this system is super patient. Potentially VERY patient, depending on whether a backhoe has cut through that OC3 in the midwest, or Vladimir Putin has chopped another submarine cable.

The physical reality around us doesn't have centralized arbiters of truth, It's decentralized.
When I set down my glass on the table, it doesn't have to coordinate with a datacenter in Ashburn to avoid spontaneously jumping to the opposite side of the table. It has local, *causal*, **coordination free** consistency.

So too, should our systems.
