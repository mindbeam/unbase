
var slab_cls   = require('./slab');
var record_cls = require('./record');
var mesh_cls   = require('./mesh');

var mesh       = new mesh_cls();

var slab1 = new slab_cls({ id: "A", mesh: mesh });
var slab2 = new slab_cls({ id: "B", mesh: mesh });
var slab3 = new slab_cls({ id: "C", mesh: mesh });



var rec1 = record_cls.createRecord(slab1, { });


console.log("\n\nInitial peering states:");
console.log( 'slab[' + slab1.id + '] peering state for item[' + rec1.id + ']', slab1.getPeeringsForItem(rec1) );
console.log( 'slab[' + slab2.id + '] peering state for item[' + rec1.id + ']', slab2.getPeeringsForItem(rec1) );
console.log( 'slab[' + slab3.id + '] peering state for item[' + rec1.id + ']', slab3.getPeeringsForItem(rec1) );

setTimeout(function(){
    
    console.log("\n\nPeering states after 1.1s:");
    console.log( 'slab[' + slab1.id + '] peering state for item[' + rec1.id + ']', slab1.getPeeringsForItem(rec1) );
    console.log( 'slab[' + slab2.id + '] peering state for item[' + rec1.id + ']', slab2.getPeeringsForItem(rec1) );
    console.log( 'slab[' + slab3.id + '] peering state for item[' + rec1.id + ']', slab3.getPeeringsForItem(rec1) );

},1100);


//var i = 10,g;
//while(i--){
//   var rec1 = record_cls.createRecord(slab1, { });
   //console.log( slab1 );

//   console.log( rec1.id, slab1.getPeeringsForItem(rec1) );
//   console.log( rec1.id, slab2.getPeeringsForItem(rec1) );
   
//   var rec2 = record_cls.createRecord(slab1, { $parent: rec1 });
   
   
//   console.log( rec2.id, slab1.getPeeringsForItem(rec2) );
//   console.log( rec2.id, slab2.getPeeringsForItem(rec2) );
//}

//slab1.killItem( rec1 );

//   console.log( rec1.id, slab2.getPeeringsForItem(rec1) );

//console.log( 'Slab 1 size', slab1.size, slab1);//.dumpItemIds().join(',') );
//console.log( 'Slab 2 size', slab2.size, slab2);//.dumpItemIds().join(',') );
//console.log( 'Slab 3 size', slab3.size, slab3.dumpItemIds().join(',') );

//testrecord1 = slab1.getItem('A01-4');
//testrecord2 = slab1.getItem('A01-5');




// Records are not "stored" anywhere
// Memos are stored. Peering necessary for count, retrieval
// index nodes participate in peering for referenced records
