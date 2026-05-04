// https://docs.rs/async-std/latest/src/async_std/future/pending.rs.html

use core::marker::PhantomData;
use core::task::{Context, Poll};
use core::pin::Pin;

pub async fn pending<T>() -> T {
    let fut = Pending {
        _marker: PhantomData,
    };
    fut.await
}

struct Pending<T> {
    _marker: PhantomData<T>,
}

impl<T> Future for Pending<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<T> {
        Poll::Pending
    }
}