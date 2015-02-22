var grain_cls = require('./grain');

var slab_increment = 0;
function Slab(args) {

  if(slab_increment > 1296)                 throw "cannot create more than 1296 slabs";
  if(typeof args != 'object')               throw "must provide args";
  if(!args.node || !args.node.length == 8)  throw "must provide 8 digit node id";
  if(!args.mesh)                            throw "must provide mesh object";

  // encode and zerofill the slab id
  this.id = args.node + ( "00" + (slab_increment++).toString(36)).substr(-2,2);
  console.log('Initialized Slab', this.id);
  if(this.id.length != 10) throw "sanity error " + this.id;

  this.grain_increment = 0;
  this._registry = {};

  this._grain_quota = 50;

  this.mesh = args.mesh;
  this.mesh.register_slab( this );

}
//Slab.prototype._registry = {};

Slab.prototype.enforceQuota = function() {
    
}

Slab.prototype.isAccepting = function() {
   var ct = this._grain_quota - Object.keys( this._registry ).length;
   return ct > 1;
}

Slab.prototype.acceptGrain = function(g) {
    if(! g instanceof grain_cls ) throw "not valid grain";

    this._registry[g.id] = g;
    console.log( 'Slab ', this.id, 'accepted grain', g.id );
};

Slab.prototype.new_grain = function(vals,cb) {
    var me = this,
        id = this.id + (this.grain_increment++).toString(36),
        g = new grain_cls(id,vals)
    ;

    this._registry[id] = g;
    this.push_grain( g, cb );

    return g;
}

Slab.prototype.push_grain = function(g,cb){
    var me     = this,
        rep_ct = g.desiredReplicas(),
        ap     = this.mesh.get_accepting_peers( this, rep_ct );

    ap.forEach(function(peer){
        me.mesh.push_grain( peer, g );
        rep_ct--;
    });

    if( rep_ct > 0 ) console.error( "unable to achieve required replica count" );
    if(cb) cb( rep_ct == 0 );

    this.enforceQuota();

};

Slab.prototype.evict_grain = function(id){
    var me = this;
    if( id instanceof grain_cls ) id = id.id;

    var g = this._registry[id];
    console.log( 'Evicting grain', id );

    this.push_grain( g, function( success ){
        if( success ){
            console.log( 'Successfully evicted grain', id );
            delete me._registry[id];
        }else{
            console.log( 'Failed to evict grain', id );
        }
    });

    return g;
}

Slab.prototype.get_grain = function(id,cb){
    var g = this._registry[id];
    if( g ) cb(g);
}

module.exports = Slab;
