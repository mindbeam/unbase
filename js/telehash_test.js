

var th = require("telehash");
var fs = require("fs");

var idfh = process.argv[2];
console.log('reading', idfh);
var id = fs.readFileSync(idfh);
if(id){
    id = JSON.parse(id);
}
console.log('id loaded');

th.mesh({id:id}, function(err, mesh){
        if(err) return console.log("mesh failed to initialize",err);
        // use mesh.* now
        console.log('mesh initialized');
        ready(mesh);
});


function ready( mesh ){
    mesh.extending({link:function(link){
      link.status(function(err){
        console.log('extending', link.hashname.substr(0,8),err?'down':'up',err||'');
      });
    }});

}
