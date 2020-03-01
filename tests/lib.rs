// Copyright 2020 nytopop (Eric Izoita)
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
use async_ctx::Context;
use tokio::time::{timeout, Duration};

const JIFFY: Duration = Duration::from_millis(10);

#[tokio::test]
async fn is_pending_if_not_completed() {
    let ctx = Context::default();
    let fut = timeout(JIFFY, ctx);

    fut.await.unwrap_err();
}

#[tokio::test]
async fn is_ready_if_completed() {
    let ctx = Context::default();
    let fut = timeout(JIFFY, ctx.clone());
    ctx.complete();

    fut.await.unwrap();
}

#[tokio::test]
async fn is_pending_if_guard_is_live() {
    let ctx = Context::default();
    let _guard = ctx.guard();
    let fut = timeout(JIFFY, ctx);

    fut.await.unwrap_err();
}

#[tokio::test]
async fn is_ready_if_guard_is_dropped() {
    let ctx = Context::default();
    let guard = ctx.guard();
    let fut = timeout(JIFFY, ctx);
    drop(guard);

    fut.await.unwrap();
}

#[tokio::test]
async fn parent_completion_propagates_to_child() {
    let ctx = Context::default();
    let chd = timeout(JIFFY, ctx.child());
    let par = timeout(JIFFY, ctx.clone());
    ctx.complete();

    par.await.unwrap();
    chd.await.unwrap();
}

#[tokio::test]
async fn child_completion_doesnt_propagate_to_parent() {
    let ctx = Context::default();
    let chd = ctx.child();
    let par = timeout(JIFFY, ctx);
    let fst = timeout(JIFFY, chd.clone());
    chd.complete();

    fst.await.unwrap();
    par.await.unwrap_err();
}
