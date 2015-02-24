
var slab_cls = require('./slab');
var mesh     = new (require('./mesh'))();

//var slab1 = new slab_cls({node: "00000001", mesh: mesh });
//var slab2 = new slab_cls({node: "00000001", mesh: mesh });

// Temporarily allowing node id to be one character for ease of reading
var slab1 = new slab_cls({node: "A", mesh: mesh });
var slab2 = new slab_cls({node: "A", mesh: mesh });
var slab3 = new slab_cls({node: "A", mesh: mesh });

var i = 10,g;
while(i--){
    g = slab1.newGrain({ some_string: "meow", $parent: g });
}
//slab1.evictGrain( g );

console.log( 'Slab 1 size', slab1.size, slab1.dumpGrainIds().join(',') );
console.log( 'Slab 2 size', slab2.size, slab2.dumpGrainIds().join(',') );
console.log( 'Slab 3 size', slab3.size, slab3.dumpGrainIds().join(',') );

console.log('A01-4 before:', slab2.getGrain('A01-4').v);

testgrain1 = slab1.getGrain('A01-4');
testgrain2 = slab1.getGrain('A01-5');

slab1.editGrain(testgrain1,{ "some_string": "woof"  });
slab1.editGrain(testgrain2,{ "some_string": "quack" });

console.log('A01-4 after:', slab2.getGrain('A01-4').v);