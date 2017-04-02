@0xb01dfd5318263fb2

struct Memo {
  id @0         : u64;
  subject_id @1 : u64;
  parents @3    : List(MemoRef)
  body : union {
  }
}

struct MemoRef {
  name @0 :Text;
  node @1 :Node;
}
