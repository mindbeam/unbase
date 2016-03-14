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

});
