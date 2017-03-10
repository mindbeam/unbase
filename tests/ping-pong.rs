extern crate unbase;
use unbase::subject::Subject;
use unbase::error::*;

#[test]
fn ping_pong() {

    /* Uncomment this, and hunt around the other test cases for the bits you need

    let simulator = unbase::network::Simulator::new();
    let net = unbase::Network::new( &simulator );

    let slab_a = unbase::Slab::new(&net);
    let slab_b = unbase::Slab::new(&net);

    let context_a = slab_a.create_context();
    let context_b = slab_b.create_context();

    let rec_a1 = Subject::new_kv(&context_a, "animal_sound", "Moo").unwrap();
    let rec_id = rec_a1.id; // useful for cross-context retrieval
    */

    // ************************************************************************
    // Your Mission, should you choose to accept it:
    // Create one record, then spawn two threads,
    // each of which makes an edit whenever it sees an edit

    // NOTE: You will have to use polling for now to detect when the
    // subject has changed, as push notification isn't implemented yet :)
    // (This is expected to be implemented by April)
    // ************************************************************************

    // spawn thread 1
        // use the original copy of the subject, or look it up by sub
        // poll for edits
        // set a value when a change is detected
    //

    // spawn thread 2
        // same as above
    //

    // Remember, we're simulating the universe here, so we have to tell the universe to continuously move forward


}
