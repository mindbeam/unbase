
var slab_cls   = require('../lib/slab');
var record_cls = require('../lib/record');
var mesh_cls   = require('../lib/mesh');

var mesh       = new mesh_cls({ min_network_latency_ms: 10, randomize_network_latency: true });

var slab1 = new slab_cls({ id: "A", mesh: mesh });
var slab2 = new slab_cls({ id: "B", mesh: mesh });
var slab3 = new slab_cls({ id: "C", mesh: mesh });

mesh.


var rec1 = record_cls.create(slab1, { animal_sound: 'moo' });

console.log( 'The value is', rec1.get('animal_sound'), rec1.getHeadMemoIDs(), rec1.getMemoIDs() );
rec1.set({ animal_sound: "woof" });


setTimeout(function(){

slab2.getRecord(rec1.id,function(rec1b){
    if (rec1b){
        console.log( 'The value is (B): ', rec1b.get('animal_sound'), rec1b.getHeadMemoIDs(), rec1b.getMemoIDs() );
        rec1b.set({ animal_sound: "meow" });
        console.log( 'The value is (B): ', rec1b.get('animal_sound'), rec1b.getHeadMemoIDs(), rec1b.getMemoIDs() );
        console.log( 'The value is (after B ): ', rec1.get('animal_sound'), rec1.getHeadMemoIDs(), rec1.getMemoIDs() );
        rec1.set({ animal_size: "large" });
        rec1b.set({ animal_size: "small" });
        console.log( 'The value is (after B ): ', rec1.get('animal_sound'),rec1.get('animal_size'), rec1.getHeadMemoIDs(), rec1.getMemoIDs() );
        console.log( 'The value is (after B ): ', rec1b.get('animal_sound'),rec1b.get('animal_size'), rec1b.getHeadMemoIDs(), rec1b.getMemoIDs() );

        setTimeout(function(){
            console.log( 'The value is (after B2): ', rec1.get('animal_sound'),rec1.get('animal_size'), rec1.getHeadMemoIDs(), rec1.getMemoIDs() );

        },1100);
    }else{
        console.log('The value is: B record not found');
    }
});

},1100);

console.log( 'The value is', rec1.get('animal_sound'), rec1.getHeadMemoIDs(), rec1.getMemoIDs() );


// console.log( slab1,slab2 );


//var i = 10,g;
//while(i--){
//   var rec1 = record_cls.createRecord(slab1, { });
   //console.log( slab1 );

//   console.log( rec1.id, slab1.getPeeringsForMemo(rec1) );
//   console.log( rec1.id, slab2.getPeeringsForMemo(rec1) );

//   var rec2 = record_cls.createRecord(slab1, { $parent: rec1 });


//   console.log( rec2.id, slab1.getPeeringsForMemo(rec2) );
//   console.log( rec2.id, slab2.getPeeringsForMemo(rec2) );
//}

//slab1.killMemo( rec1 );

//   console.log( rec1.id, slab2.getPeeringsForMemo(rec1) );

//console.log( 'Slab 1 size', slab1.size, slab1);//.dumpMemoIds().join(',') );
//console.log( 'Slab 2 size', slab2.size, slab2);//.dumpMemoIds().join(',') );
//console.log( 'Slab 3 size', slab3.size, slab3.dumpMemoIds().join(',') );

//testrecord1 = slab1.getMemo('A01-4');
//testrecord2 = slab1.getMemo('A01-5');




// Records are not "stored" anywhere
// Memos are stored. Peering necessary for count, retrieval
// index nodes participate in peering for referenced records
