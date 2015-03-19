
# What is Unbase?

Unbase is a concept for a database+application framework that is fundamentally reactive, fault tolerant, and decentralized.
It seeks to address some very specific shortcomings in traditional database+application paradigms.
Unbase is presently under active design.

## Why
The Unbase concept is an attempt to create a truly distributed architecture that transcends device, geography, programming language,
and present orthodoxy about what constitutes a "database". It seeks to blur the lines between application/database, and client/server.

When you're looking to scale, there are a couple ways to achieve this:

### Build a bunker
Big iron, a large diesel generator, maybe some thick concrete walls, some guards; and a moat perhaps, and an RDBMS
But you still have to do business in Beijing, Sydney, Seattle, Buenos Aires, and Kansas City... what about latency? Ok, five bunkers then. (Hmm, sounds expensive)

Classical relational databases are inherently poor when it comes to decentralization.
Multi-master has never been a byword for simplicity, nor will it magically solve all your operational problems.

Some [modern](http://www.nuodb.com/) [databases](http://www.clustrix.com/) are trying to solve this, but they are largely closed-source offerings trying to gain
market-share by pretending to be an old big-iron database, but just a bit more scalable. They may be successful in this endeavor, but the solution they offer is far from holistic.
They play nice with existing software stacks, offer a familiar SQL interface, solve a few problems, and they leave lots more unresolved.

* You must connect to them - Even if it's a process on the same machine, you must still copy the data into your process over the loopback.
* Your process has to poll for updates - they're not pushed to you.
* They lack integral queuing and push notifications. If another party wants to hear about updates, you must propagate these changes twice - Once for DB replication, and again via an out-of-band message queue.
 * Parallel replication streams race each other toward event consumers, who lack the practical ability to synchronize them
   

### Spread out
Guerilla warefare style. Go forth into homes, and into businesses. Bring the data to the people. Get up close and personal.

Centralization has too many shortcomings to ignore.
Distributed systems, carefully implemented, are the key to achieving higher speed, scalability, and fault tolerance.
We need a better answer to service the demands of modern software. Humans can do this, why can't software?
We need to build in delegation, compromise, and regionality from the start.

This is what Unbase seeks to achieve.

## How

With Unbase, there is effectively no distinction between application and database.
The application is *inside* the database. The database simply happens to lack a single "Base"

In the abstract, this is actually a very old idea. Stored procedures have been around for quite some time, and databases used
to be more like application servers. A user would call a stored procedure, and business logic happens.
Here, the same principle applies, it's just a bit more spread out.

## Design Goals:

* Efficient push notifications for all changes to all interested parties
* Drastically reduce latency by moving data close to where you need it [closer to the processor][what_is_distance]
* Allow continued operation during a network partition
 * Cheat CAP theorem limitations by *selectively* loosening consistency guarantees (with informed consent)
 * Treat consistency violations as inevitable, and allow them to be systematically resolved
* Destroy the distinction between client and server. They are considered identical **except** for policy, capability, and resources.
 * Access control enforcement at every stage of replication
 * Push business logic to initiators when possible, otherwise delegate to nearest capable node
* Support for complex data types to limit unnecessary entity-attribute-value structures
* Virtualized objects, accessible from any node, complete with synchronous, asynchronous business logic enforcement

## Design Non-Goals:

* Serializability - [Fundimentally incompatible](https://groups.google.com/forum/#!msg/cloud-computing/nn7Sw5T0eSE/NxOTUwD_0ykJ) with distributed systems.
* SQL support     - SQL might be added at a later date, but only as a means of introspection / administration. It
