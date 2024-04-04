#![no_std]
#![no_main]

mod flow;

use core::pin::Pin;
use embassy_time::Duration;
use embassy_time::Timer;

use embassy_executor::Spawner;
use embassy_stm32::init;
use flow::{Flow, FlowCollector, FlowHandle2};
use pinned_init::*;
use singleton_cell::new_singleton;
use singleton_cell::SCell;
use {defmt_rtt as _, panic_probe as _};

new_singleton!(pub ListToken);
new_singleton!(pub DataToken);

enum Event {
    Event,
}

struct Context<'c> {
    counter: u32,
    x: &'c mut u32,
}

struct ComponentData(u32);

struct ComponentHandler1<'a>(&'a SCell<DataToken, ComponentData>);

impl<'a> ComponentHandler1<'a> {
    fn extra(&mut self, token: &mut DataToken) {
        self.0.borrow_mut(token).0 += 1;
        defmt::info!("Handler1 extra");
    }
}

struct ComponentHandler2<'a>(&'a SCell<DataToken, ComponentData>);

impl<'a> ComponentHandler2<'a> {
    fn extra(&mut self, token: &mut DataToken) {
        self.0.borrow_mut(token).0 -= 1;
        defmt::info!("Handler2 extra");
    }
}

impl<'c, 'a> FlowCollector<Event, Context<'c>> for ComponentHandler1<'a> {
    fn emit(&mut self, _value: &Event, context: &mut Context) {
        context.counter += 2;
        *context.x += 1;
    }
}

impl<'c, 'a> FlowCollector<Event, Context<'c>> for ComponentHandler2<'a> {
    fn emit(&mut self, _value: &Event, context: &mut Context) {
        context.counter -= 1;
        *context.x += 1;
    }
}

#[pin_data]
struct Component<'h, 'c> {
    #[pin]
    pub handle: FlowHandle2<
        'h,
        ListToken,
        DataToken,
        Context<'c>,
        ComponentData,
        Event,
        ComponentHandler1<'h>,
        Event,
        ComponentHandler2<'h>,
    >,
}

impl<'h, 'c> Component<'h, 'c> {
    fn new<'f>(flow: Pin<&'f Flow<'h, ListToken, Event, Context<'c>>>) -> impl PinInit<Self> + 'f
    where
        'c: 'h,
    {
        pin_init!(Self {
            handle <- FlowHandle2::new(
                flow.as_ref(), flow.as_ref(), ComponentData(0), |cell| ComponentHandler1(&cell), |cell| ComponentHandler2(&cell)
            )
        })
    }
}

#[embassy_executor::main]
async fn main(mut _spawner: Spawner) {
    let _p = init(Default::default());

    let mut list_token = ListToken::new().expect("only created once!");
    let mut data_token = DataToken::new().expect("only created once!");
    let mut x: u32 = 0;
    let mut context = Context {
        counter: 0,
        x: &mut x,
    };

    stack_pin_init!(let flow = Flow::<ListToken, Event, Context>::new());
    stack_pin_init!(let component = Component::new(flow.as_ref()));

    loop {
        flow.emit(&mut list_token, &Event::Event, &mut context);
        component
            .handle
            .handle1
            .borrow_mut(&mut list_token)
            .extra(&mut data_token);
        component
            .handle
            .handle2
            .borrow_mut(&mut list_token)
            .extra(&mut data_token);
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
    }
}
