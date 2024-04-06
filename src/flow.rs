use core::cell::Cell;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::ptr;
use singleton_cell::SCell;

/// Invariant: the `prev` and `next` pointers must form a fully consistent
/// doubly-linked _cycle_ / ring.
#[::pin_project::pin_project(PinnedDrop)]
pub
struct RingNodeLinks<'h, V, Cxt> {
    prev: Cell<ptr::NonNull<dyn RingNode<'h, V, Cxt>>>,
    next: Cell<ptr::NonNull<dyn RingNode<'h, V, Cxt>>>,
    #[pin]
    _pin_sensitive: ::core::marker::PhantomPinned,
}

impl<'h, V, Cxt> RingNodeLinks<'h, V, Cxt> {
    fn is_uninit(&self) -> bool {
        ::core::ptr::eq(
            self.prev.get().as_ptr().cast(),
            &DANGLING_NODE,
        )
    }

    fn detach(self: Pin<&Self>) {
        if self.is_uninit() {
            return;
        }
        // if already "empty": self <-> self.
        if self.prev.get().as_ptr().cast() as *const Self == &*self as _ {
            return;
        }
        // otherwise, prev <-> self <-> next (where `prev` and `next` may be the same link).
        let (prev_links, next_links) = unsafe {
            // # SAFETY
            //
            // The invariant of our `Ring{,Node}`s stipulates that any `RingNode`
            // in it, thanks to the `Pin`ning invariants, will, sequentially
            // (no `Sync` thus no parallelism), remove themselves from the cycle
            // on drop, before any of these other pointers get the chance to dangle.
            //
            // Thus, any pointer in a `RingNode` is guaranteed not to dangle.
            (
                self.prev().node_links(),
                self.next().node_links(),
            )
        };

        prev_links.next.set(self.next.get());
        next_links.prev.set(self.prev.get());
    }
}

#[::pin_project::pinned_drop]
impl<'h, V, Cxt> PinnedDrop for RingNodeLinks<'h, V, Cxt> {
    fn drop(self: Pin<&mut Self>) {
        self.as_ref().detach()
    }
}

impl<'h, V, Cxt> RingNode<'h, V, Cxt> for RingNodeLinks<'h, V, Cxt> {
    fn node_links(self: Pin<&Self>) -> Pin<&Self> {
        self
    }
}

pub
trait RingNode<'h, V, Cxt, __ = &'h (V, Cxt)> : RingNodeExt<'h, V, Cxt, __> {
    fn node_links(self: Pin<&Self>) -> Pin<&RingNodeLinks<'h, V, Cxt>>;

    fn collector_mut(self: Pin<&mut Self>)
      -> Option<Pin<&mut dyn RingCollector<V, Cxt>>>
    {
        // default impl
        None
    }
}

pub
trait RingNodeExt<'h, V, Cxt, __ = &'h (V, Cxt)> : 'h {
    unsafe
    fn prev(self: Pin<&Self>)
      -> Pin<&dyn RingNode<'h, V, Cxt>>
    ;

    unsafe
    fn next(self: Pin<&Self>)
      -> Pin<&dyn RingNode<'h, V, Cxt>>
    ;

    fn insert_before(
        self: Pin<&Self>,
        target_node: Pin<&RingNodeLinks<'h, V, Cxt>>,
    )
    ;
}

impl<'h, V, Cxt, T> RingNodeExt<'h, V, Cxt> for T
where
    Self : RingNode<'h, V, Cxt>,
{
    unsafe
    fn prev(self: Pin<&Self>)
      -> Pin<&dyn RingNode<'h, V, Cxt>>
    {
        // Safety: the pointer must be valid as guaranteed by the caller.
        // Pin::new_unchecked() is fine since all nodes have been inserted
        // once witnessed behind a `Pin`.
        Pin::new_unchecked(self.node_links().prev.get().as_ref())
    }

    unsafe
    fn next(self: Pin<&Self>)
      -> Pin<&dyn RingNode<'h, V, Cxt>>
    {
        // Safety: the pointer must be valid as guaranteed by the caller.
        // Pin::new_unchecked() is fine since all nodes have been inserted
        // once witnessed behind a `Pin`.
        Pin::new_unchecked(self.node_links().next.get().as_ref())
    }

    fn insert_before(
        self: Pin<&Self>,
        target_node: Pin<&RingNodeLinks<'h, V, Cxt>>,
    )
    {
        let this_ptr = ptr::NonNull::from(&*self);
        let this_node = self.node_links();

        this_node.detach(); // in case of calling `insert_into()` multiple times.

        let prev = unsafe {
            // # SAFETY
            //
            // Since `Ring` is cyclic, the pointer is always valid.
            // The resulting reference is only temporarily used and not store so we do
            // not have any aliasing issues.
            target_node.prev().node_links()
        };

        this_node.next.set(       prev.next.replace(this_ptr));
        this_node.prev.set(target_node.prev.replace(this_ptr));
    }
}

impl<V, Cxt> RingNodeLinks<'_, V, Cxt> {
    fn dangling() -> Self {
        Self {
            prev: Cell::new(ptr::NonNull::from(&DANGLING_NODE)),
            next: Cell::new(ptr::NonNull::from(&DANGLING_NODE)),
            _pin_sensitive: <_>::default(),
        }
    }

    pub
    fn init(self: Pin<&Self>) {
        let this = ptr::NonNull::<Self>::from(&*self);
        self.prev.set(this);
        self.next.set(this);
    }
}

#[::pin_project::pin_project]
/// A `RingNode` which has been given usufruct (full `&mut` powers) over
/// all of its nodes.
pub
struct Ring<'h, V, Cxt> {
    #[pin]
    root: RingNodeLinks<'h, V, Cxt>,
    sauron: Sauron,
}

impl<V, Cxt> Ring<'_, V, Cxt> {
    pub fn new() -> Self {
        Self {
            root: RingNodeLinks::dangling(),
            sauron: Sauron(()),
        }
    }

    pub
    fn init(self: Pin<&Self>) {
        let this = self.project_ref();
        this.root.init();
    }
}

impl<'h, V, Cxt> RingNode<'h, V, Cxt> for Ring<'h, V, Cxt> {
    fn node_links(self: Pin<&Self>) -> Pin<&RingNodeLinks<'h, V, Cxt>> {
        self.project_ref().root
    }
}

/// Token to help manipulate a `Ring`'s nodes in a more checked fashion.
///
/// The ideä is [`Sauron`] owns its [`Ring`], and through it, all of ring
/// servants.
///
/// That is, _via_ <code>&'input mut [Sauron]</code> access, one will be able to
/// have <code>&'input mut dyn [IsRingNode]<...></code> access, or shorter-lived
/// reborrows thereof.
///
/// Mainly, consider the following mistake:
/// While holding `&mut root`, let's get ahold of `&mut *root.next`.
/// While doing that, let's get ahold of `&mut *next.prev = &mut root`.
/// We now have two overlapping `&mut`s.
///
/// Now, using the Sauron token:
/// While holding `&root`, we may temporarily `&mut`-upgrade it to `&mut root`
/// via `&mut sauron`, but while this `&mut root` is held, we can no longer
/// upgrade any other node.
struct Sauron(());

impl Sauron {
    /// Safety: the `ptr` must be part of this Sauron's ring nodes
    unsafe
    fn to_ref<RingNode : ?Sized>(&self, ptr: ptr::NonNull<RingNode>)
      -> Pin<&RingNode>
    {
        // Safety: ptrs in the ring have already been witnessed as `Pin`ned.
        Pin::new_unchecked(ptr.as_ref())
    }

    /// Safety: the `ptr` must be part of this Sauron's ring nodes.
    unsafe
    fn to_mut<RingNode : ?Sized>(&mut self, ptr: ptr::NonNull<RingNode>)
      -> Pin<&mut RingNode>
    {
        // Safety: ptrs in the ring have already been witnessed as `Pin`ned.
        Pin::new_unchecked({ ptr }.as_mut())
    }
}

pub trait RingCollector<V, Cxt> {
    fn emit(self: Pin<&mut Self>, value: &V, context: &mut Cxt);
}

impl<'h, V, Cxt> RingCollector<V, Cxt> for Ring<'h, V, Cxt> {
    fn emit(mut self: Pin<&mut Self>, value: &V, context: &mut Cxt) {
        assert!(!self.root.is_uninit());

        let root_ptr: ptr::NonNull<dyn RingNode<V, Cxt>> =
            ptr::NonNull::from(unsafe { self.as_mut().get_unchecked_mut() })
        ;
        let sauron = self.as_mut().project().sauron;
        let mut cursor: ptr::NonNull<dyn RingNode<V, Cxt>> = root_ptr;
        loop {
            let cur_node: Pin<&dyn RingNode<V, Cxt>> = unsafe {
                // # SAFETY
                //
                // We've checked against `uninit`.
                // Thus, the pointer cannot be dangling.
                sauron.to_ref(cursor)
            };

            let next_node_ptr = cur_node.node_links().next.get();

            if ::core::ptr::addr_eq(next_node_ptr.as_ptr(), root_ptr.as_ptr()) {
                return;
            }

            let next_node_mut = unsafe {
                // # SAFETY
                //
                // Since `Ring` is cyclic, the pointer is never undefined.
                // It always points to a valid `Handle` and not to `Ring`'s root due to the previous check.
                // All `Handle` pointers have been created through mutable references in contrast to the root Ring pointer,
                // so we can safely cast to mut.
                //
                // We also need to ensure that not only the trait objects but also any references contained in their corresponding NodeHandles
                // are not alive during this method call. This is ensured through the borrowed token.

                sauron.to_mut(next_node_ptr)
            };

            if let Some(collector) = next_node_mut.collector_mut() {
                collector.emit(value, context);
            } else {
                // should this be an error?
            }

            cursor = next_node_ptr;
        }
    }
}

#[::pin_project::pin_project]
pub struct RingNodeWithData<'h, V, Cxt, T>
where
    T: 'h + RingCollector<V, Cxt>,
{
    #[pin]
    node: RingNodeLinks<'h, V, Cxt>,

    #[pin]
    data: T,
}

impl<'h, V, Cxt, T> RingNodeWithData<'h, V, Cxt, T>
where
    T: 'h + RingCollector<V, Cxt>,
{
    pub
    fn new(data: T) -> Self {
        Self {
            node: RingNodeLinks::dangling(),
            data,
        }
    }
}

impl<'h, V, Cxt, T> RingNode<'h, V, Cxt>
    for RingNodeWithData<'h, V, Cxt, T>
where
    T: 'h + RingCollector<V, Cxt>,
{
    fn node_links(self: Pin<&Self>) -> Pin<&RingNodeLinks<'h, V, Cxt>> {
        self.project_ref().node
    }

    fn collector_mut(self: Pin<&mut Self>)
      -> Option<Pin<&mut dyn RingCollector<V, Cxt>>>
    {
        Some(self.project().data)
    }
}

impl<K, T: RingCollector<V, Cxt>, V, Cxt>
    RingCollector<V, Cxt>
for
    SCell<K, T>
where
    // we need `Unpin` since the `SCell` type is missing a pin-projection
    // getter, which forces us to go through `&mut`.
    T : Unpin,
{
    fn emit(self: Pin<&mut Self>, value: &V, context: &mut Cxt) {
        Pin::new(self.get_mut().get_mut()).emit(value, context)
    }
}

impl<'h, V, Cxt, T> Deref for RingNodeWithData<'h, V, Cxt, T>
where
    T: 'h + RingCollector<V, Cxt>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.data
    }
}

impl<'h, V, Cxt, T> DerefMut for RingNodeWithData<'h, V, Cxt, T>
where
    T: 'h + Unpin + RingCollector<V, Cxt>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<'h, V, Cxt, T> RingNodeWithData<'h, V, Cxt, T>
where
    T: 'h + RingCollector<V, Cxt>,
{
    pub
    fn data_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        self.project().data
    }
}

struct DanglingNode(());
static DANGLING_NODE: DanglingNode = DanglingNode(());

impl<'h, V, Cxt> RingNode<'h, V, Cxt> for DanglingNode {
    fn node_links(self: Pin<&Self>) -> Pin<&RingNodeLinks<'h, V, Cxt>> {
        unimplemented!("use of dangling node, i.e., uninit link!");
    }
}
