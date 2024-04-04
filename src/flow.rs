use core::cell::Cell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::ptr::{addr_of, NonNull};
use pinned_init::*;
use singleton_cell::{SCell, Singleton};

pub trait FlowCollector<V, Cxt> {
    fn emit(&mut self, value: &V, context: &mut Cxt);
}

trait IsFlowNode<'h, V, Cxt> {
    fn node_ptrs(&self) -> &FlowNodePtrs<'h, V, Cxt>;
    fn collector(&mut self) -> &mut dyn FlowCollector<V, Cxt>;
}

struct FlowNodePtrs<'h, V, Cxt> {
    prev: Cell<NonNull<dyn IsFlowNode<'h, V, Cxt> + 'h>>,
    next: Cell<NonNull<dyn IsFlowNode<'h, V, Cxt> + 'h>>,
}

#[pin_data]
pub struct Flow<'h, K, V, Cxt> {
    root: FlowNodePtrs<'h, V, Cxt>,
    _phantom: PhantomData<K>,
}

impl<'h, K, V, Cxt> Flow<'h, K, V, Cxt> {
    pub fn new() -> impl PinInit<Self>
    where
        V: 'h,
        K: 'h,
        Cxt: 'h,
    {
        pin_init!(&this in Flow {
            root: FlowNodePtrs {
                prev: Cell::new(this),
                next: Cell::new(this),
            },
            _phantom: PhantomData
        })
    }

    pub fn emit(&self, _token: &mut K, value: &V, context: &mut Cxt)
    where
        K: Singleton,
    {
        let root_ptr: NonNull<dyn IsFlowNode<V, Cxt> + '_> = NonNull::from(self);
        let mut prev_node_ptr = root_ptr;
        loop {
            let prev_node = unsafe {
                // # SAFETY
                //
                // At this point, the pointer points either to a valid `Handle` or to `Flow`'s root and as such is valid to dereference.
                // We need to ensure that the reference to the trait object is unique which is the
                // case since it is only acquired by this function and is otherwise private.

                prev_node_ptr.as_ref()
            };

            let mut emit_node_ptr = prev_node.node_ptrs().next.get();

            if emit_node_ptr == root_ptr {
                return;
            }

            let emit_node = unsafe {
                // # SAFETY
                //
                // Since `Flow` is cyclic, the pointer is never undefined.
                // It always points to a valid `Handle` and not to `Flow`'s root due to the previous check.
                // All `Handle` pointers have been created through mutable references in contrast to the root Flow pointer,
                // so we can safely cast to mut.
                //
                // We also need to ensure that not only the trait objects but also any references contained in their corresponding NodeHandles
                // are not alive during this method call. This is ensured through the borrowed token.

                emit_node_ptr.as_mut()
            };

            emit_node.collector().emit(value, context);

            prev_node_ptr = emit_node_ptr;
        }
    }
}

impl<'h, K, V, Cxt> IsFlowNode<'h, V, Cxt> for Flow<'h, K, V, Cxt> {
    fn node_ptrs(&self) -> &FlowNodePtrs<'h, V, Cxt> {
        &self.root
    }
    fn collector(&mut self) -> &mut dyn FlowCollector<V, Cxt> {
        unreachable!("Collector should never be called through emit");
    }
}

#[pin_data(PinnedDrop)]
pub struct FlowHandle<'h, K, V, Cxt, T> {
    node_ptrs: FlowNodePtrs<'h, V, Cxt>,
    collector: SCell<K, T>,
}

impl<'h, K, V, Cxt, T> Deref for FlowHandle<'h, K, V, Cxt, T> {
    type Target = SCell<K, T>;
    fn deref(&self) -> &Self::Target {
        &self.collector
    }
}

impl<'h, K, V, Cxt, T> DerefMut for FlowHandle<'h, K, V, Cxt, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.collector
    }
}

impl<'h, K, V, Cxt, T> IsFlowNode<'h, V, Cxt> for FlowHandle<'h, K, V, Cxt, T>
where
    T: FlowCollector<V, Cxt>,
{
    fn node_ptrs(&self) -> &FlowNodePtrs<'h, V, Cxt> {
        &self.node_ptrs
    }
    fn collector(&mut self) -> &mut dyn FlowCollector<V, Cxt> {
        self.collector.get_mut()
    }
}

impl<'h, K, V, Cxt, T> FlowHandle<'h, K, V, Cxt, T> {
    pub fn new<'f>(flow: Pin<&'f Flow<'h, K, V, Cxt>>, collector: T) -> impl PinInit<Self> + 'f
    where
        V: 'h,
        K: 'h,
        Cxt: 'h,
        T: FlowCollector<V, Cxt> + 'h + Unpin,
    {
        pin_init!(&this in Self{
            node_ptrs: {
                let prev = {
                    let prev = unsafe {

                        // # SAFETY
                        //
                        // Since `Flow` is cyclic, the pointer is always valid.
                        // The resulting reference is only temporarily used and not store so we do
                        // not have any aliasing issues.

                        flow.root.prev.get().as_ref()
                    };
                    prev.node_ptrs()
                };

                FlowNodePtrs {
                    prev: Cell::new(flow.root.prev.replace(this)),
                    next: Cell::new(prev.next.replace(this))
                }
            },
            collector: SCell::new(collector)
        })
    }
}

#[pinned_drop]
impl<'h, K, V, Cxt, T> PinnedDrop for FlowHandle<'h, K, V, Cxt, T> {
    fn drop(self: Pin<&mut Self>) {
        let (prev_ptrs, next_ptrs) = {
            let (prev_ptrs, next_ptrs) = unsafe {
                // # SAFETY
                //
                // `Flow` outlives all handles. Any non-alive handles will have unsubscribed by now.
                // All remaining pointers point to valid handles due to the cyclic nature of `Flow`.
                // We never store references to the next & prev 's pointees and since they are private, at no point are two references alive.

                (
                    self.node_ptrs.prev.get().as_ref(),
                    self.node_ptrs.next.get().as_ref(),
                )
            };
            (prev_ptrs.node_ptrs(), next_ptrs.node_ptrs())
        };

        prev_ptrs.next.set(self.node_ptrs.next.get());
        next_ptrs.prev.set(self.node_ptrs.prev.get());
    }
}

#[pin_data]
pub struct FlowHandle2<'h, K1, K2, Cxt, T, V1, H1, V2, H2> {
    pub data: SCell<K2, T>,
    #[pin]
    pub handle1: FlowHandle<'h, K1, V1, Cxt, H1>,
    #[pin]
    pub handle2: FlowHandle<'h, K1, V2, Cxt, H2>,
}

impl<'h, K1, K2, Cxt, T, V1, H1, V2, H2> FlowHandle2<'h, K1, K2, Cxt, T, V1, H1, V2, H2> {
    pub fn new<'f, F1, F2>(
        flow1: Pin<&'f Flow<'h, K1, V1, Cxt>>,
        flow2: Pin<&'f Flow<'h, K1, V2, Cxt>>,
        data: T,
        collector1_builder: F1,
        collector2_builder: F2,
    ) -> impl PinInit<Self> + 'f
    where
        V1: 'h,
        V2: 'h,
        K1: 'h,
        K2: 'h,
        Cxt: 'h,
        T: 'h,
        F1: FnOnce(&'h SCell<K2, T>) -> H1 + 'f,
        F2: FnOnce(&'h SCell<K2, T>) -> H2 + 'f,
        H1: FlowCollector<V1, Cxt> + 'h + Unpin,
        H2: FlowCollector<V2, Cxt> + 'h + Unpin,
    {
        pin_init!(&this in Self {
                    data: SCell::new(data),
                    handle1 <-FlowHandle::new(flow1, collector1_builder(
                                        unsafe {&*addr_of!((*this.as_ptr()).data)})),
                    handle2 <-FlowHandle::new(flow2, collector2_builder(
                                        unsafe {&*addr_of!((*this.as_ptr()).data)})),
        })
    }
}

#[pin_data]
pub struct FlowHandle3<'h, K1, K2, Cxt, T, V1, H1, V2, H2, V3, H3> {
    pub data: SCell<K2, T>,
    #[pin]
    pub handle1: FlowHandle<'h, K1, V1, Cxt, H1>,
    #[pin]
    pub handle2: FlowHandle<'h, K1, V2, Cxt, H2>,
    #[pin]
    pub handle3: FlowHandle<'h, K1, V3, Cxt, H3>,
}

impl<'h, K1, K2, Cxt, T, V1, H1, V2, H2, V3, H3>
    FlowHandle3<'h, K1, K2, Cxt, T, V1, H1, V2, H2, V3, H3>
{
    pub fn new<'f, F1, F2, F3>(
        flow1: Pin<&'f Flow<'h, K1, V1, Cxt>>,
        flow2: Pin<&'f Flow<'h, K1, V2, Cxt>>,
        flow3: Pin<&'f Flow<'h, K1, V3, Cxt>>,
        data: T,
        collector1_builder: F1,
        collector2_builder: F2,
        collector3_builder: F3,
    ) -> impl PinInit<Self> + 'f
    where
        V1: 'h,
        V2: 'h,
        V3: 'h,
        K1: 'h,
        K2: 'h,
        Cxt: 'h,
        T: 'h,
        F1: FnOnce(&'h SCell<K2, T>) -> H1 + 'f,
        F2: FnOnce(&'h SCell<K2, T>) -> H2 + 'f,
        F3: FnOnce(&'h SCell<K2, T>) -> H3 + 'f,
        H1: FlowCollector<V1, Cxt> + 'h + Unpin,
        H2: FlowCollector<V2, Cxt> + 'h + Unpin,
        H3: FlowCollector<V3, Cxt> + 'h + Unpin,
    {
        pin_init!(&this in Self {
                    data: SCell::new(data),
                    handle1 <-FlowHandle::new(flow1, collector1_builder(
                                        unsafe {&*addr_of!((*this.as_ptr()).data)})),
                    handle2 <-FlowHandle::new(flow2, collector2_builder(
                                        unsafe {&*addr_of!((*this.as_ptr()).data)})),
                    handle3 <-FlowHandle::new(flow3, collector3_builder(
                                        unsafe {&*addr_of!((*this.as_ptr()).data)})),
        })
    }
}
