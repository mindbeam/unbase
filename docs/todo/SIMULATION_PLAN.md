#. Using a naive 3d spatial coordinate system: create, and randomly assign a position to N slabs.
##. Create a single "seed" slab at a random point in space.
##. For each slab created, recurse to create M slabs using the previously created slab as its seed.
##. New slabs should be a random, yet bounded distance from the seed slab
##. Stop when the total number of slabs created = N
##. possible modification of this approach: bind slab coordinates to those on the surface of an ~13k km sphere, and ensure network paths follow the curvature of said sphere
# For each slab, store:
##. a local clock, starting at 0
##. a probability of this local clock ticking in a given cycle?
##. spatial coordinates
##. probability distribution over time for intentional termination
##. probability distribution over time for hardware failure (necessary to differentiate?)
##. probability for random message send failures
##. probability for random message receive failures
##. a "user type" which
###. activates different sets of behaviors
###. associates a series of intervals of activity/inactivity (coordinated to earth civil time or otherwise)
#. create various editing behaviors, each with a probability associated with a given slab "user type"
##. indecision – alternate edits back and forth a bounded number of times
##. swoop and tweak – randomly select a record against which to make a small number of edits.
##. back to back – make a rapid series of edits to different aspects of a topic, leaving some sub-minute gap between each
##. *add a few more*
#. Perform the simulation in rounds. For each round:
#. Each round shall interrogate each slab once for behaviors which may be triggered (probabilistically)
#. Memos which are to be sent to another slab are queued
