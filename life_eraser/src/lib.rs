#![no_std]
#![feature(type_alias_impl_trait)]

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    pin::Pin,
    future::Future
};

use core::marker::PhantomData;

pub trait LifetimeEraserMap<'s>
where
    Self: 's,
{
    type Owner;
    type Borrowed<'a>
    where
        Self::Owner: 's;
}

struct LifetimeEraser<'s, T: LifetimeEraserMap<'s>> {
    fptr: for<'a> fn(&'a mut T::Owner) -> T::Borrowed<'a>,
    data: MaybeUninit<T::Borrowed<'s>>,
}

struct LifetimeEraserHandle<'s, 'a, T: LifetimeEraserMap<'s>>(&'a mut MaybeUninit<T::Borrowed<'s>>);

impl<'s, 'a, T: LifetimeEraserMap<'s>> Deref for LifetimeEraserHandle<'s, 'a, T> {
    type Target = T::Borrowed<'a>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            core::mem::transmute::<&MaybeUninit<T::Borrowed<'s>>, &MaybeUninit<T::Borrowed<'a>>>(
                self.0,
            )
            .assume_init_ref()
        }
    }
}

impl<'s, 'a, T: LifetimeEraserMap<'s>> DerefMut for LifetimeEraserHandle<'s, 'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::mem::transmute::<
                &mut MaybeUninit<T::Borrowed<'s>>,
                &mut MaybeUninit<T::Borrowed<'a>>,
            >(self.0)
            .assume_init_mut()
        }
    }
}

impl<'s, 'a, T: LifetimeEraserMap<'s>> Drop for LifetimeEraserHandle<'s, 'a, T> {
    fn drop(&mut self) {
        unsafe {
            core::mem::transmute::<
                &mut MaybeUninit<T::Borrowed<'s>>,
                &mut MaybeUninit<T::Borrowed<'a>>,
            >(self.0)
            .assume_init_drop();
        }
    }
}

impl<'s, 'a, T: LifetimeEraserMap<'s>> Future for LifetimeEraserHandle<'s, 'a, T>
where
    for<'c> T::Borrowed<'c>: Future + 'c,
{
    type Output = <T::Borrowed<'a> as Future>::Output;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let m: &mut <T as LifetimeEraserMap>::Borrowed<'_> = self.deref_mut();
        let m: Pin<&mut dyn Future<Output = Self::Output>> = unsafe { Pin::new_unchecked(m) };
        m.poll(cx)
    }
}

impl<'s, T: LifetimeEraserMap<'s>> LifetimeEraser<'s, T> {
    pub fn new(fptr: for<'a> fn(&'a mut T::Owner) -> T::Borrowed<'a>) -> Self {
        Self {
            fptr,
            data: MaybeUninit::uninit(),
        }
    }

    pub fn erase<'a>(&'a mut self, x: &'a mut T::Owner) -> LifetimeEraserHandle<'s, 'a, T>
    where
        T::Owner: 'a,
    {
        (unsafe {
            core::mem::transmute::<
                &mut MaybeUninit<T::Borrowed<'s>>,
                &mut MaybeUninit<T::Borrowed<'a>>,
            >(&mut self.data)
        })
        .write((self.fptr)(x));
        LifetimeEraserHandle(&mut self.data)
    }
}

struct ErasedLifetimeErasureMap;
impl<'s> LifetimeEraserMap<'s> for ErasedLifetimeErasureMap {
    type Owner = ();
    type Borrowed<'a> = ();
}

type ErasedHandle<'s, 'a> = LifetimeEraserHandle<'s, 'a, ErasedLifetimeErasureMap>;

pub struct LifetimeErasedFutureHandle<'s, 'a, T> {
    _erased: ErasedHandle<'s, 'a>,
    _drop: fn(&mut ErasedHandle<'s, 'a>),
    _phantom: PhantomData<T>,
}

impl<'s, 'a, T> Drop for LifetimeErasedFutureHandle<'s, 'a, T> {
    fn drop(&mut self) {
        (self._drop)(&mut self._erased)
    }
}

impl<'s, 'a, T> Future for LifetimeErasedFutureHandle<'s, 'a, T> {
    type Output = T;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let m: Pin<&mut (dyn Future<Output = T> + '_)> = self.as_mut();
        m.poll(cx)
    }
}

impl<'s, 'a, T: LifetimeEraserMap<'s>, R> From<LifetimeEraserHandle<'s, 'a, T>>
    for LifetimeErasedFutureHandle<'s, 'a, R>
where
    T::Borrowed<'a>: Future<Output = R> + 'a,
{
    fn from(value: LifetimeEraserHandle<'s, 'a, T>) -> Self {
        Self {
            _erased: unsafe {
                core::mem::transmute::<LifetimeEraserHandle<'s, 'a, T>, ErasedHandle<'s, 'a>>(value)
            },
            _drop: |h| unsafe {
                core::mem::transmute::<
                    *mut ErasedHandle<'s, 'a>,
                    *mut LifetimeEraserHandle<'s, 'a, T>,
                >(h as *mut ErasedHandle<'s, 'a>)
                .drop_in_place()
            },
            _phantom: PhantomData,
        }
    }
}

// EXAMPLE USAGE

// ORIGINAL TRAIT

pub trait AsyncFn {
    async fn test_a(&mut self) -> ();
    async fn test_b(&mut self) -> ();
}

// ADAPTED TRAIT

pub trait ObjectSafeAsyncFn<'s> {
    fn test_a<'a>(&'a mut self) -> LifetimeErasedFutureHandle<'s, 'a, ()>;
    fn test_b<'a>(&'a mut self) -> LifetimeErasedFutureHandle<'s, 'a, ()>;
}

type FnTestAMethod<'a, T: AsyncFn> = impl Future<Output = ()>;
fn method_a_wrapper<'a, T: AsyncFn>(x: &'a mut T) -> FnTestAMethod<'a, T> {
    AsyncFn::test_a(x)
}

type FnTestBMethod<'a, T: AsyncFn> = impl Future<Output = ()>;
fn method_b_wrapper<'a, T: AsyncFn>(x: &'a mut T) -> FnTestBMethod<'a, T> {
    AsyncFn::test_b(x)
}

struct ObjectSafeAsyncFnTestAMap<T>(PhantomData<T>);
struct ObjectSafeAsyncFnTestBMap<T>(PhantomData<T>);

impl<'s, T: AsyncFn + 's> LifetimeEraserMap<'s> for ObjectSafeAsyncFnTestAMap<T> {
    type Owner = T;
    type Borrowed<'a> = FnTestAMethod<'a, T>;
}

impl<'s, T: AsyncFn + 's> LifetimeEraserMap<'s> for ObjectSafeAsyncFnTestBMap<T> {
    type Owner = T;
    type Borrowed<'a> = FnTestBMethod<'a, T>;
}

struct ObjectSafeAsyncFnWrapper<'s, T: AsyncFn> {
    wrapped: T,
    test_a_container: LifetimeEraser<'s, ObjectSafeAsyncFnTestAMap<T>>,
    test_b_container: LifetimeEraser<'s, ObjectSafeAsyncFnTestBMap<T>>,
}

impl<'s, T: AsyncFn> ObjectSafeAsyncFnWrapper<'s, T> {
    fn new(wrapped: T) -> Self {
        Self {
            wrapped,
            test_a_container: LifetimeEraser::new(method_a_wrapper),
            test_b_container: LifetimeEraser::new(method_b_wrapper),
        }
    }
}

impl<'s, T: AsyncFn> ObjectSafeAsyncFn<'s> for ObjectSafeAsyncFnWrapper<'s, T> {
    fn test_a<'a>(&'a mut self) -> LifetimeErasedFutureHandle<'s, 'a, ()> {
        self.test_a_container.erase(&mut self.wrapped).into()
    }

    fn test_b<'a>(&'a mut self) -> LifetimeErasedFutureHandle<'s, 'a, ()> {
        self.test_b_container.erase(&mut self.wrapped).into()
    }
}

struct Example<'a>(&'a mut u32, PhantomData<&'a ()>);
impl<'a> AsyncFn for Example<'a> {
    async fn test_a(&mut self) -> () {
        ()
    }

    async fn test_b(&mut self) -> () {
        {
            *self.0 += 1;
            ()
        }
    }
}

async fn test() {
    let mut x = 0;
    let example = Example(&mut x, PhantomData);
    let mut example = ObjectSafeAsyncFnWrapper::new(example);
    example.test_a().await;
    example.test_b().await;
}
