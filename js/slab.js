var grain = require('./grain');

var slab_increment = 0;
function Slab(args) {

  if(slab_increment > 1296) throw "cannot create more than 1296 slabs";
  if(typeof args != 'object') throw "must provide args object";
  if(!args.node || !args.node.length == 8) throw "must provide 8 digit node id";

  // encode and zerofill the slab id
  this.id = args.node + ( "00" + (slab_increment++).toString(36)).substr(-2,2);
  console.log('Initialized Slab', this.id);
  if(this.id.length != 10) throw "sanity error " + this.id;

  this.grain_increment = 0;
  this._registry = {};
}
//Slab.prototype._registry = {};

Slab.prototype.add_grain = function() {
    
};

Slab.prototype.new_grain = function(vals) {
    var id = this.id + (this.grain_increment++).toString(36);
    var g = new grain(id,vals);

    this._registry[id] = g;
    return g;
};

module.exports = Slab;
