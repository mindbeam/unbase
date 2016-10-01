---
layout: page
title: "Consistency Model"
category: design
seq: 2
---


Having an explicitly-stated consistency model is very important for a modern computational system.
This allows users to set their expectations, and reason about the behavior of the system.

We are calling the Unbase consistency model **Infectious Knowledge.**

  
---  
  

#### Potential Causality
*Infectious Knowledge* is similar to Potential Causality, insofar as it intends to guarantee that all potential causations are accounted for when projecting state for a given observer. The main difference is that under the _Infectious Knowledge_ model, the system is willfully ignorant of some concurrent causal threads which may be inside of the receiving light cone. These causal threads are assimilated on an as-needed basis, rather than an immediate basis.

#### Why?
This means no shared state, no linearizability, no quorums (except for those you choose to implement as an overlay).
This is believed to be the strongest consistency guarantees which are possible without coordination under presently understood physical laws, and which is feasible on modern computing hardware.

TODO:
* Expand on differences, explain the process of knowledge infection.
* Link to takedown of Linearizable-first systems
