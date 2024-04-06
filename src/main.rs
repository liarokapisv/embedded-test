#![no_std]
#![no_main]

mod flow;

use core::pin::{Pin, pin};
// use embassy_stm32::pac::sai::vals::Comp;
use embassy_time::Duration;
use embassy_time::Timer;

use embassy_executor::Spawner;
use embassy_stm32::init;
use flow::{Ring, RingCollector, RingNode, RingNodeExt, RingNodeWithData};
use {defmt_rtt as _, panic_probe as _};

enum Event {
    Event,
}

struct Context<'c> {
    counter: u32,
    x: &'c mut u32,
}

struct Handler1();

impl Handler1 {
    fn extra(&mut self) {
        defmt::info!("Handler1 extra");
    }
}

struct Handler2();

impl Handler2 {
    fn extra(&mut self) {
        defmt::info!("Handler2 extra");
    }
}

impl<'c> RingCollector<Event, Context<'c>> for Handler1 {
    fn emit(self: Pin<&mut Self>, _value: &Event, context: &mut Context) {
        context.counter += 2;
        *context.x += 1;
    }
}

impl<'c> RingCollector<Event, Context<'c>> for Handler2 {
    fn emit(self: Pin<&mut Self>, _value: &Event, context: &mut Context) {
        context.counter -= 1;
        *context.x += 1;
    }
}

#[::pin_project::pin_project]
struct ComponentA<'h, 'c> {
    #[pin]
    handle1: RingNodeWithData<'h, Event, Context<'c>, Handler1>,
    #[pin]
    handle2: RingNodeWithData<'h, Event, Context<'c>, Handler2>,
}

impl<'h, 'c> ComponentA<'h, 'c> {
    fn new() -> Self {
        Self {
            handle1: RingNodeWithData::new(Handler1()),
            handle2: RingNodeWithData::new(Handler2()),
        }
    }

    fn init<'f>(
        self: Pin<&Self>,
        ring: Pin<&'f Ring<'h, Event, Context<'c>>>,
    )
    {
        let ring_node = ring.node_links();
        let this = self.project_ref();
        this.handle1.insert_before(ring_node.as_ref());
        this.handle2.insert_before(ring_node);
    }
}

#[::pin_project::pin_project]
struct ComponentB<'h, 'c> {
    #[pin]
    comp1: ComponentA<'h, 'c>,
    #[pin]
    comp2: ComponentA<'h, 'c>,
}

impl<'h, 'c> ComponentB<'h, 'c> {
    fn new() -> Self {
        Self {
            comp1: ComponentA::new(),
            comp2: ComponentA::new(),
        }
    }

    fn init(
        self: Pin<&Self>,
        ring: Pin<&Ring<'h, Event, Context<'c>>>,
    )
    {
        let this = self.project_ref();
        this.comp1.init(ring.as_ref());
        this.comp2.init(ring.as_ref());
    }
}

#[::embassy_executor::main]
async fn main(mut _spawner: Spawner) {
    let _p = init(Default::default());

    // let mut token = Token::new().expect("only created once!");
    let mut x: u32 = 0;
    let mut context = Context {
        counter: 0,
        x: &mut x,
    };

    let mut flow = pin!(Ring::<Event, Context>::new());
    flow.as_ref().init();
    let mut component = pin!(ComponentB::new());
    component.as_ref().init(flow.as_ref());

    loop {
        flow.as_mut().emit(&Event::Event, &mut context);
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        component.as_mut().project().comp1.project().handle1.data_mut().extra();
        component.as_mut().project().comp1.project().handle2.data_mut().extra();
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        component.as_mut().project().comp1.project().handle1.data_mut().extra();
        component.as_mut().project().comp1.project().handle2.data_mut().extra();
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
    }
}
