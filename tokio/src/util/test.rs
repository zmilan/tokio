use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

pub(crate) fn with_timeout(duration: Duration, f: impl FnOnce() + Send + 'static) {
    let (done_tx, done_rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        f();

        // Send a message on the channel so that the test thread can
        // determine if we have entered an infinite loop:
        done_tx.send(()).unwrap();
    });

    // Since the failure mode of this test is an infinite loop, rather than
    // something we can easily make assertions about, we'll run it in a
    // thread. When the test thread finishes, it will send a message on a
    // channel to this thread. We'll wait for that message with a fairly
    // generous timeout, and if we don't recieve it, we assume the test
    // thread has hung.
    //
    // Note that it should definitely complete in under a minute, but just
    // in case CI is slow, we'll give it a long timeout.
    match done_rx.recv_timeout(duration) {
        Err(RecvTimeoutError::Timeout) => panic!(
            "test did not complete within {:.2} seconds, \
             we have (probably) entered an infinite loop!",
            duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
        ),
        // Did the test thread panic? We'll find out for sure when we `join`
        // with it.
        Err(RecvTimeoutError::Disconnected) => {
            println!("done_rx dropped, did the test thread panic?");
        }
        // Test completed successfully!
        Ok(()) => {}
    }

    thread.join().expect("test thread should not panic!")
}
