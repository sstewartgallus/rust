// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Internal module so doen't need to be documented
#[allow(missing_doc)];

/**
 * An extremely primitive wait queue concurrency abstraction.
 *
 * Maybe this can be exposed publicly if it is deemed useful, or good
 * enough but for now it is a private implementation detail of some of
 * our concurrency tools.
 */

use std::comm::SendDeferred;
use std::comm;


type WaitEnd = comm::PortOne<()>;
type SignalEnd = comm::ChanOne<()>;


pub struct WaitEvent { priv wait_end: WaitEnd }

impl WaitEvent {
    pub fn wait(self) {
        let WaitEvent { wait_end } = self;
        comm::recv_one(wait_end)
    }
}


/// A doubly-ended queue of waiting tasks.
pub struct WaitQueue {
    priv head: comm::Port<SignalEnd>,
    priv tail: comm::Chan<SignalEnd>
}

impl WaitQueue {
    pub fn new() -> WaitQueue {
        let (block_head, block_tail) = comm::stream();
        WaitQueue { head: block_head, tail: block_tail }
    }

    // Signals one live task from the queue.
    pub fn signal(&self) -> bool {
        // The peek is mandatory to make sure recv doesn't block.
        if self.head.peek() {
            // Pop and send a wakeup signal. If the waiter was killed, its port
            // will have closed. Keep trying until we get a live task.
            if self.head.recv().try_send_deferred(()) {
                true
            } else {
                self.signal()
            }
        } else {
            false
        }
    }

    pub fn broadcast(&self) -> uint {
        let mut count = 0;
        while self.head.peek() {
            if self.head.recv().try_send_deferred(()) {
                count += 1;
            }
        }
        count
    }

    pub fn queue_wait_event(&self) -> WaitEvent {
        let (wait_end, signal_end) = comm::oneshot();
        self.tail.send_deferred(signal_end);
        WaitEvent { wait_end: wait_end }
    }
}
