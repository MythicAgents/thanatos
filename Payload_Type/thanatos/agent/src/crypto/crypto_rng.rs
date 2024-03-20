use std::rc::Rc;

use super::rng::Rng;

thread_local! {
    // Use an RC so that each access to the thread local RNG refers to the same
    // RNG handle instead of creating a new RNG handle for every access.
    //
    // This also prevents the RNG handle from being accidentally destroyed
    // while it is being borrowed.
    static THREAD_RNG: Rc<Rng> = {
        let rng = Rng::new().unwrap_or_else(|_| panic!("TLS RNG failed to initialize"));
        Rc::new(rng)
    }
}

/// Returns a reference to the current thread's CSPRNG
pub fn thread_rng() -> Rc<Rng> {
    THREAD_RNG.with(|tls_rng| tls_rng.clone())
}
