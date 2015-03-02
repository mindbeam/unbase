
# What is Unbase?

Unbase is a concept for a database+application framework that is fundimentally reactive, fault tolerant, and decentralized.
It seeks to address some very specific shortcomings in traditional database+application paradigms.
Unbase is presently under active design.

## Why
The Unbase concept is an attempt to create a truly distributed architecture that trancends device, geography, programming language,
and present orthodoxy about what constitutes a "database". It seeks to blur the lines between application/database, and client/server.

Classical relational databases are inherantly poor when it comes to decentralization.
(Multi-master has never been a buy-word for simplicity, nor will it magically solve all your operational problems.)

Some [modern](http://www.nuodb.com/) [databases](http://www.clustrix.com/) are trying to solve this, but they are trying primarily to gain
marketshare by pretending to be old-fasioned database. They may be successful in this endeavor, but the solution they offer is far from holistic.
They play nice with existing software stacks, offer a familiar SQL interface, solve a few problems, and leave lots more unresolved.



* You must connect to them to transact business - Even if it's a process on the same machine, it's still far slower than if it were in-process
* They lack integral queueing and push notifications. You must propagate these changes twice if your aim is to be reactive
 * Once for the DB replication itself, and again via an out-of-band message queue.
* Parallel replication streams race each other toward event consumers, who lack the practical ability to synchronize them

## How

You can't argue with CAP theorem, that's settled. All the same, centralization has too many shortcomings to ignore.
We need a better answer to service the demands of modern software. Humans can do this, why can't software?
We need to build in delegation, compromise, and regionality from the start.

With Unbase, there is effectively no distinction between application and database.
The application is *inside* the database. The database simply happens to lack a single "Base"

In the abstract, this is actually a very old idea. Stored procedures have been around for quite some time, and databases used
to be more like application servers. A user would call a stored procedure, and business logic happens.
Here, the same principal applies, it's just a bit more spread out.

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
* Virtualized objects, accessable from any node, complete with synchronous, asynchronous business logic enforcement

## Design Non-Goals:

* Serializability - [Fundimentally incompatible](https://groups.google.com/forum/#!msg/cloud-computing/nn7Sw5T0eSE/NxOTUwD_0ykJ) with distributed systems.
* SQL support     - SQL might be added at a later date, but only as a means of introspection / administration. It