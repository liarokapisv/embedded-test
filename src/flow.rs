use core::mem::MaybeUninit;
use core::pin::Pin;

pub trait FlowCollector<V, Cxt> {
    fn emit(&mut self, value: &mut V, context: &mut Cxt);
}

pub struct Flow<V, Cxt>(FlowNodePtrs<V, Cxt>);

pub trait IsFlowNode<V, Cxt> {
    fn node_ptrs(&mut self) -> &mut FlowNodePtrs<V, Cxt>;
    fn collector(&mut self) -> &mut dyn FlowCollector<V, Cxt>;
}

pub struct FlowNodePtrs<V, Cxt> {
    prev: *mut dyn IsFlowNode<V, Cxt>,
    next: *mut dyn IsFlowNode<V, Cxt>,
}

impl<V, Cxt> IsFlowNode<V, Cxt> for Flow<V, Cxt> {
    fn node_ptrs(&mut self) -> &mut FlowNodePtrs<V, Cxt> {
        &mut self.0
    }
    fn collector(&mut self) -> &mut dyn FlowCollector<V, Cxt> {
        self
    }
}

impl<V, Cxt> FlowCollector<V, Cxt> for Flow<V, Cxt> {
    fn emit(&mut self, value: &mut V, context: &mut Cxt) {
        let root: *mut dyn IsFlowNode<V, Cxt> = self;
        let mut node = root;
        loop {
            // SAFETY:
            // It is always valid to dereference `node` since it is always pointing to
            // a valid node.
            node = unsafe { (*node).node_ptrs().next };
            if core::ptr::addr_eq(node, root) {
                return;
            }
            // SAFETY:
            // node->next always points to a proper node since the root node points to itself
            // when there are no collectors attached.
            (unsafe { &mut *node }).collector().emit(value, context);
        }
    }
}

pub struct FlowNodeHandle<V, Cxt, T> {
    node_ptrs: FlowNodePtrs<V, Cxt>,
    x: T,
}

impl<V, Cxt, T> IsFlowNode<V, Cxt> for FlowNodeHandle<V, Cxt, T>
where
    T: FlowCollector<V, Cxt>,
{
    fn node_ptrs(&mut self) -> &mut FlowNodePtrs<V, Cxt> {
        &mut self.node_ptrs
    }
    fn collector(&mut self) -> &mut dyn FlowCollector<V, Cxt> {
        &mut self.x
    }
}

impl<V, Cxt, T> FlowNodeHandle<V, Cxt, T> {
    pub fn new<'s, 'f: 's>(
        storage: Pin<&'s mut MaybeUninit<Self>>,
        flow: Pin<&'f mut Flow<V, Cxt>>,
        x: T,
    ) -> Pin<&'s mut Self>
    where
        V: 'static,
        Cxt: 'static,
        T: FlowCollector<V, Cxt> + 'static + Unpin,
    {
        let node = (unsafe { storage.get_unchecked_mut() }).write(FlowNodeHandle {
            x,
            node_ptrs: FlowNodePtrs {
                prev: flow.0.prev,
                next: unsafe { flow.get_unchecked_mut() },
            },
        });
        unsafe {
            (*node.node_ptrs.next).node_ptrs().prev = node;
            (*node.node_ptrs.prev).node_ptrs().next = node;
        }
        Pin::new(node)
    }
}

impl<'a, V, Cxt, T> Drop for FlowNodeHandle<V, Cxt, T> {
    fn drop(&mut self) {
        unsafe {
            (*self.node_ptrs.prev).node_ptrs().next = self.node_ptrs.next;
            (*self.node_ptrs.next).node_ptrs().prev = self.node_ptrs.prev;
        }
    }
}
