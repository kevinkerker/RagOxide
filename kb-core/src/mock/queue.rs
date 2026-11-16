//! First-in-first-out queue of answers for the [`MockAnswerer`](super::MockAnswerer).

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Poll, Waker};

#[derive(Default)]
struct State {
    next: u64,
    running: Option<u64>,
    waiting: VecDeque<u64>,
    wakers: HashMap<u64, Waker>,
}

/// Queue of answers: one runs, the others wait in the order they joined.
#[derive(Default)]
pub(crate) struct TurnQueue {
    state: Mutex<State>,
}

/// A place in a [`TurnQueue`], taken when `ask` is called. While it runs it holds the
/// turn; dropping it gives the turn to the next waiting ticket, or leaves the queue
/// without changing the order of the others.
pub(crate) struct Ticket {
    queue: Arc<TurnQueue>,
    id: u64,
}

impl TurnQueue {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Joins the queue at its end; runs at once if nobody runs.
    pub(crate) fn join(queue: &Arc<Self>) -> Ticket {
        let mut state = queue.lock();
        let id = state.next;
        state.next += 1;
        if state.running.is_none() {
            state.running = Some(id);
        } else {
            state.waiting.push_back(id);
        }
        Ticket {
            queue: Arc::clone(queue),
            id,
        }
    }
}

impl Ticket {
    /// `true` while this ticket holds the turn.
    pub(crate) fn runs(&self) -> bool {
        self.queue.lock().running == Some(self.id)
    }

    /// Waits until this ticket holds the turn.
    pub(crate) async fn turn(&self) {
        futures::future::poll_fn(|cx| {
            let mut state = self.queue.lock();
            if state.running == Some(self.id) {
                Poll::Ready(())
            } else {
                state.wakers.insert(self.id, cx.waker().clone());
                Poll::Pending
            }
        })
        .await;
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        let next = {
            let mut state = self.queue.lock();
            state.wakers.remove(&self.id);
            if state.running == Some(self.id) {
                state.running = state.waiting.pop_front();
                state.running.and_then(|next| state.wakers.remove(&next))
            } else {
                state.waiting.retain(|id| *id != self.id);
                None
            }
        };
        // Woken after the lock is released, so a waker that calls back into the queue
        // cannot deadlock.
        if let Some(waker) = next {
            waker.wake();
        }
    }
}
