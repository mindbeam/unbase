
var slab_cls = require('./slab');
var record_cls = require('./record');
var mesh     = new (require('./mesh'))();

//var slab1 = new slab_cls({node: "00000001", mesh: mesh });
//var slab2 = new slab_cls({node: "00000001", mesh: mesh });

// Temporarily allowing node id to be one character for ease of reading
var slab1 = new slab_cls({node: "A", mesh: mesh });
var slab2 = new slab_cls({node: "A", mesh: mesh });
var slab3 = new slab_cls({node: "A", mesh: mesh });

var i = 10,g;
while(i--){
    g = new record_cls(slab1, { some_string: "meow", $parent: g });
}

//slab1.evictRecord( g );

console.log( 'Slab 1 size', slab1.size, slab1.dumpRecordIds().join(',') );
console.log( 'Slab 2 size', slab2.size, slab2.dumpRecordIds().join(',') );
console.log( 'Slab 3 size', slab3.size, slab3.dumpRecordIds().join(',') );

console.log('A01-4 before:', slab2.getRecord('A01-4').v);

testrecord1 = slab1.getRecord('A01-4');
testrecord2 = slab1.getRecord('A01-5');


slab1.editRecord(testrecord1,{ "some_string": "woof"  });
slab1.editRecord(testrecord2,{ "some_string": "quack" });

console.log('A01-4 after:', slab2.getRecord('A01-4').v);





// Records are not "stored" anywhere
// Memos are stored. Peering necessary for count, retrieval
// index nodes participate in peering for referenced records




memo1a - memo1b - memo1c


peerable 