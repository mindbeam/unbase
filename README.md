
# What is Unbase?

The idea behind Unbase is to create a framework that is fundimentally reactive, fault tolerant, and decentralized.
Unbase is presently being designed.

## Why
In the interest of addressing some very specific shortcomings in traditional application design,
Unbase is an attempt to create a truly distributed architecture that trancends device,
geography, programming language, and orthodoxy.
It seeks to blur the lines between application/database, and client/server.

Classical relational databases are inherantly poor when it comes to decentralization.
Multi-master has never been a buy-word for simplicity, nor will it magically solve all your operational problems.

Some modern RDBMSes are trying to fix this, but largely they are trying to gain marketshare by pretending to be old-fasioned database:
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

* SQL support - SQL may be added at a later date, but it should be considered only as a means of introspection / administration.