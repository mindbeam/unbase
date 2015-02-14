
var th = require("telehash");
th.generate(function(err, endpoint){
  if(err) return console.log("endpoint generation failed",err);
  // endpoint contains a `keys:{}`, `secrets:{}`, and `hashname:"..."` 
});

console.log(th);
