use core::cell::Cell;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::ptr::NonNull;
use pinned_init::*;
use singleton_cell::{SCell, Singleton};

pub trait FlowCollector<K, V, Cxt> {
    fn emit(&mut self, token: &mut K, value: &V, context: &mut Cxt);
}

trait IsFlowNode<K, V, Cxt> {
    fn node_ptrs(&self) -> &FlowNodePtrs<K, V, Cxt>;
    fn collector(&mut self) -> &mut dyn FlowCollector<K, V, Cxt>;
}

struct FlowNodePtrs<K, V, Cxt> {
    prev: Cell<NonNull<dyn IsFlowNode<K, V, Cxt>>>,
    next: Cell<NonNull<dyn IsFlowNode<K, V, Cxt>>>,
}

#[pin_data]
pub struct Flow<K, V, Cxt> {
    root: FlowNodePtrs<K, V, Cxt>,
}

impl<K, V, Cxt> Flow<K, V, Cxt> {
    pub fn new() -> impl PinInit<Self>
    where
        V: 'static,
        K: 'static,
        Cxt: 'static,
    {
        pin_init!(&this in Flow {
            root: FlowNodePtrs {
                prev: Cell::new(this),
                next: Cell::new(this),
            }
        })
    }

    pub fn emit(&self, token: &mut K, value: &V, context: &mut Cxt)
    where
        K: Singleton,
    {
        let root_ptr: NonNull<dyn IsFlowNode<K, V, Cxt>> = NonNull::from(self);
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

            emit_node.collector().emit(token, value, context);

            prev_node_ptr = emit_node_ptr;
        }
    }
}

impl<K: Singleton, V, Cxt> FlowCollector<K, V, Cxt> for Flow<K, V, Cxt> {
    fn emit(&mut self, token: &mut K, value: &V, context: &mut Cxt) {
        (self as &Self).emit(token, value, context);
    }
}

impl<K, V, Cxt> IsFlowNode<K, V, Cxt> for Flow<K, V, Cxt> {
    fn node_ptrs(&self) -> &FlowNodePtrs<K, V, Cxt> {
        &self.root
    }
    fn collector(&mut self) -> &mut dyn FlowCollector<K, V, Cxt> {
        unreachable!("Collector should never be called through emit");
    }
}

#[pin_data(PinnedDrop)]
pub struct FlowHandle<K, V, Cxt, T> {
    node_ptrs: FlowNodePtrs<K, V, Cxt>,
    x: SCell<K, T>,
}

impl<K, V, Cxt, T> Deref for FlowHandle<K, V, Cxt, T> {
    type Target = SCell<K, T>;
    fn deref(&self) -> &Self::Target {
        &self.x
    }
}

impl<K, V, Cxt, T> DerefMut for FlowHandle<K, V, Cxt, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.x
    }
}

impl<K, V, Cxt, T> IsFlowNode<K, V, Cxt> for FlowHandle<K, V, Cxt, T>
where
    T: FlowCollector<K, V, Cxt>,
{
    fn node_ptrs(&self) -> &FlowNodePtrs<K, V, Cxt> {
        &self.node_ptrs
    }
    fn collector(&mut self) -> &mut dyn FlowCollector<K, V, Cxt> {
        self.x.get_mut()
    }
}

impl<K, V, Cxt, T> FlowHandle<K, V, Cxt, T> {
    pub fn new(flow: Pin<&Flow<K, V, Cxt>>, x: T) -> impl PinInit<Self> + '_
    where
        V: 'static,
        K: 'static,
        Cxt: 'static,
        T: FlowCollector<K, V, Cxt> + 'static + Unpin,
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
            x: SCell::new(x)
        })
    }
}

#[pinned_drop]
impl<K, V, Cxt, T> PinnedDrop for FlowHandle<K, V, Cxt, T> {
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
