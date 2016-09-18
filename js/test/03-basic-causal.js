var slab_cls   = require('../lib/slab');
var record_cls = require('../lib/record');
var mesh_cls   = require('../lib/mesh');
var should     = require('should');

var mesh       = new mesh_cls({ test_mode: 1, debug: 0 });

var contextA, contextB, recA1;

describe('basic causal consistency', function() {

    it('should be initialize two slabs, and one context each', () => {
        contextA = new slab_cls({ id: "A", mesh: mesh }).createContext();
        contextB = new slab_cls({ id: "B", mesh: mesh }).createContext();
        mesh.knownSlabCount().should.be.exactly(2);
    });

    it('should create a record on context A', () => {
        recA1 = record_cls.create(contextA, { animal_sound: 'meow' });
        should(recA1).be.ok();
    });

    it('delivers all messages', () => mesh.deliverAllQueuedMessages() );

    it('should look up that record on context B', () => {
        return contextB.getRecord( recA1.id ).then( ( recB1 ) =>{
             should(recB1).be.ok();
             recB1.getFreshOrNull('animal_sound').should.be.exactly('meow');
        })
    });

    it('should originate an edit on the context A record, returning a causal reference',() => {
        recA1.set({'animal_sound': 'woof'});
    });
    // context B is outside the light cone of the above edit
    it('magically transport causal reference from context A to context B',() => {
        contextB.addRawContext( contextA.getPresentContext() );
    });

    it('should read record on context B with transported causal reference',() => {
        return contextB.getRecord( recA1.id ).then( ( recB1 ) =>{
             should(recB1).be.ok();
              // TODO: change get to getFreshOrNull
              // TODO: causal barrier should cause getFreshOrNull to respond with null
             should(recB1.getFreshOrNull('animal_sound')).not.be.ok();
        });
    });

    it('delivers all messages', () => mesh.deliverAllQueuedMessages() );


    // beacons are essentially sparse vector clocks - a compression methond for a vector clock
    // rather than representing a vector clock as a vector, let is be a set where each clocl is keyed on the id of their respective beacon
    // beacons would likely have to be originated from a common root beacon in order to guarantee that any party can ultimately materialize the relevant portion of the sparse vector clock when comparing to a formerly uknown beacon.
    // Each beacon ping would of course contain all causal context for the previous commit cycle,
    // but subsequent pings would pear back other directly referenced beacons using LRU. When a beacon
    // version becomes expunged from the active set due to LRU, it is essentially compressed, and remains
    // determinible by way traversing the former pings until the beacon in question, or the root(or contemporary?)
    // beacon is arrived at.

    // Q1: is sparse vectorclock compression scheme theoretically equivalent to a non-sparse vector clock? I think so
    // Q2: is traversal to the root beacon node necessary in very rate circumstances, or frequent circumstances?
    // Q3: were we to take a root-beacon-less approach, and instead use a number-of-hops limit, would there be any possibility of this being deterministic for other observers? ( presumably not possible )
    // Q4: confirm: beacons are essentially a way to arrive at commmon points of comparison more often than would occur with fine-grain causal reference alone

    // * for the purposes of initial testing, all nodes can be beacons. back this off using probableistic behavior later
    // * each beacon ping must reference N other beacons (suggest N=5) of the most active peers (is older causal chain info sufficient to prevent closed cycles among beacons?)
    // * create 100k slabs
    // * create a randomized workload on each slab
    // * measure the occasions of causal fencing
    // * measure the number of direct causal determinations vs indirect ( beacon traversals )


});
