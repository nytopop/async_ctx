// Copyright 2020 nytopop (Eric Izoita)
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//! Asynchronous contexts.
#![warn(rust_2018_idioms, missing_docs)]

use futures::task::AtomicWaker;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    task::{self, Poll},
};

/// A future that can be completed externally as an asynchronous cancellation mechanism.
///
/// Resolves if any of the following occur:
///
/// * [complete][Context::complete] is called
/// * a derived [Guard] is dropped
/// * a parent [Context] completes
///
/// Clones can be expected to refer to the same logical entity.
#[derive(Clone, Default)]
pub struct Context {
    parent: Option<Box<Context>>,
    cond: Arc<AtomicBool>,
    wake: Arc<AtomicWaker>,
}

impl Future for Context {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if let Some(Poll::Ready(())) = self.parent.as_mut().map(Pin::new).map(|p| p.poll(ctx)) {
            return Poll::Ready(());
        }

        self.wake.register(ctx.waker());

        if self.cond.load(Relaxed) {
            return Poll::Ready(());
        }

        Poll::Pending
    }
}

impl Context {
    /// Create a RAII guard that will [complete][Context::complete] this context (and any
    /// derived children) when the guard is dropped.
    pub fn guard(&self) -> Guard {
        Guard(self.clone())
    }

    /// Complete this context (and any derived children).
    pub fn complete(&self) {
        self.cond.store(true, Relaxed);
        self.wake.wake();
    }

    /// Derive a child context. Completion of the parent (self) will propagate to the child,
    /// but not vice-versa.
    pub fn child(&self) -> Self {
        Self {
            parent: Some(Box::new(self.clone())),
            ..Self::default()
        }
    }
}

/// A RAII guard that will [complete][Context::complete] its source context when dropped.
///
/// Holding the guard does not prevent completion from other sources.
pub struct Guard(Context);

impl Drop for Guard {
    fn drop(&mut self) {
        self.0.complete();
    }
}
