
var slab_cls   = require('../lib/slab');
var record_cls = require('../lib/record');
var mesh_cls   = require('../lib/mesh');

//var assert = require('chai').assert;
var should = require('should');

describe('eventual-consistency', function() {

    var mesh       = new mesh_cls({ disconnected: 1 });

    it('mesh should be an object', () => {
        // assert.typeOf(mesh, 'object','mesh is an object');
        mesh.should.be.instanceof(Object);
    });

    var slabA = new slab_cls({ id: "A", mesh: mesh });
    var slabB = new slab_cls({ id: "B", mesh: mesh });
    var slabC = new slab_cls({ id: "C", mesh: mesh });

    it('should be correctly configured', () => {
        mesh.knownSlabCount().should.be.exactly(3);
    });

    var recA1 = record_cls.create(slabA, { animal_sound: 'moo' });

    it('new record should be internally consistent', () => {
        recA1.get('animal_sound').should.equal('moo');
    });

    it('new record should not yet have conveyed to slab B', (done) => {
        slabB.getRecord( recA1.id ).then( function ( recB1 ) {
            // shouldn't have made its way to slabB yet
            should(recB1).not.be.ok();
            done();
        }).catch((err) => console.error(err) );

    });

    it('fast forward time a bit', () => mesh.deliverAllQueuedMessages() );

    it('new record should now be available on slab B', () => {
        return slabB.getRecord( recA1.id ).then( ( recB1 ) => {
            should(recB1).be.ok();
            recB1.get('animal_sound').should.be.exactly('moo');
        });
    });

    it('new record should now be available on slab C', () => {
        return slabC.getRecord( recA1.id ).then( ( recC1 ) => {
            should(recC1).be.ok();
            recC1.get('animal_sound').should.be.exactly('moo');
        });
    });

});
