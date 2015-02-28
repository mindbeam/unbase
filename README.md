
# What is PanDaemon

The idea behind PanDaemon is to create a framework that is fundimentally reactive, fault tolerant, and decentralized.

In the interest of addressing some very specific shortcomings in traditional application design,
PanDaemon is an attempt to create a truly distributed architecture that trancends device,
geography, programming language, and orthodoxy.
It seeks to blur the lines between application/database, and client/server.

Classical RDBMS Databases are bad at decentralization. Multi-master wasn't a buy-word for simplicity ( and it still isn't )
Centralization is a recipe for disappointment, and is incompatible with maturning demands on hosted software systems.

Some modern RDBMSes are trying to fix this, but largely they are trying to gain marketshare by pretending
to be old-fasioned database:
* You must connect to them to transact business - Even if it's a process on the same machine, it's still orders of magnitude slower than if it were in-process)
* They lack integral queueing and push notifications
* You must propagate these changes twice: Once for the DB replication itself, and again via an out-of-band message queue.
* Parallel replication streams race each other toward event consumers, who lack the practical ability to synchronize them

Under the PanDaemon design, there would be no distinction between application and database.
The application IS the database, and the database IS the application.

This is actually a very old idea: Stored Procedures have been around for quite some time.
Love them or hate them, traditional stored procedures are simultaneously awesome, and terrible.
They're awesome because they embed application logic inside the database, where business logic can be easily enforced, and reactively deployed.
They're TERRIBLE because they're usually written in some arcane language, they're terribly difficult to call, and they don't scale as well as other approaches.

PanDaemon Design goals:
* House your entire application (not just your data)
* Drastically, drasticaly reduce latency.
* Dramatically improve reliability, redundancy, and fault tolerance.
* Destroy the distinction between system where the there is fundimentally no distinction
between client and server, except as is defined by policy and resources.



* Treate consistency enforcementfailure simply as a record edit


# Possible Names

* Baseless
* Unbase
* Unbased
* AllYourBase



