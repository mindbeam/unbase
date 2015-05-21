
var slab_cls   = require('./slab');
var record_cls = require('./record');
var mesh       = new (require('./mesh'))();

//var slab1 = new slab_cls({node: "00000001", mesh: mesh });
//var slab2 = new slab_cls({node: "00000001", mesh: mesh });

// Temporarily allowing node id to be one character for ease of reading
var slab1 = new slab_cls({node: "A", mesh: mesh });
var slab2 = new slab_cls({node: "A", mesh: mesh });
//var slab3 = new slab_cls({node: "A", mesh: mesh });

//var i = 10,g;
//while(i--){
   var rec1 = record_cls.createRecord(slab1, { });
   //console.log( slab1 );
   
   console.log( rec1.id, slab1.getPeeringsForItem(rec1) );
   console.log( rec1.id, slab2.getPeeringsForItem(rec1) );
   
   var rec2 = record_cls.createRecord(slab1, { $parent: rec1 });
   
   
   console.log( rec2.id, slab1.getPeeringsForItem(rec2) );
   console.log( rec2.id, slab2.getPeeringsForItem(rec2) );
//}

//slab1.evictItem( rec1 );

//console.log( 'Slab 1 size', slab1.size, slab1);//.dumpItemIds().join(',') );
//console.log( 'Slab 2 size', slab2.size, slab2);//.dumpItemIds().join(',') );
//console.log( 'Slab 3 size', slab3.size, slab3.dumpItemIds().join(',') );

//testrecord1 = slab1.getItem('A01-4');
//testrecord2 = slab1.getItem('A01-5');




// Records are not "stored" anywhere
// Memos are stored. Peering necessary for count, retrieval
// index nodes participate in peering for referenced records
