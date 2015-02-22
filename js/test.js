
var slab_cls = require('./slab');
var mesh     = new (require('./mesh'))();

var slab1 = new slab_cls({node: "00000001", mesh: mesh });
var slab2 = new slab_cls({node: "00000001", mesh: mesh });


var i = 100,g;
while(i--){
    g = slab1.newGrain({ $parent: g });

//    slab1.evict_grain( g );
}

console.log( g );
