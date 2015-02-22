

var slab = new (require('./slab'))({node: "00000001"});

var i = 1000000,g;
while(i--){
    g = slab.new_grain({ parent: g });
}

console.log( g );
