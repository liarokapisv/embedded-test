#![no_std]
#![no_main]

mod flow;

use core::pin::Pin;
use embassy_time::Duration;
use embassy_time::Timer;

use embassy_executor::Spawner;
use embassy_stm32::init;
use flow::{Flow, FlowCollector, FlowHandle};
use pinned_init::*;
use singleton_cell::new_singleton;
use {defmt_rtt as _, panic_probe as _};

new_singleton!(pub Token);

enum Event {
    Event,
}

struct Context {
    counter: u32,
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

impl FlowCollector<Event, Context> for Handler1 {
    fn emit(&mut self, _value: &Event, context: &mut Context) {
        context.counter += 2;
    }
}

impl FlowCollector<Event, Context> for Handler2 {
    fn emit(&mut self, _value: &Event, context: &mut Context) {
        context.counter -= 1;
    }
}

#[pin_data]
struct Component {
    #[pin]
    handle1: FlowHandle<Token, Event, Context, Handler1>,
    #[pin]
    handle2: FlowHandle<Token, Event, Context, Handler2>,
}

impl Component {
    fn new(flow: Pin<&Flow<Token, Event, Context>>) -> impl PinInit<Self> + '_ {
        pin_init!(Self {
            handle1 <- FlowHandle::new(flow, Handler1()),
            handle2 <- FlowHandle::new(flow, Handler2())
        })
    }
}

#[embassy_executor::main]
async fn main(mut _spawner: Spawner) {
    let _p = init(Default::default());

    let mut token = Token::new().expect("only created once!");

    stack_pin_init!(let flow = Flow::<Token, Event, Context>::new());
    stack_pin_init!(let component = Component::new(flow.as_ref()));

    let mut context = Context { counter: 0 };

    loop {
        flow.emit(&mut token, &Event::Event, &mut context);
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        component.handle1.borrow_mut(&mut token).extra();
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        component.handle2.borrow_mut(&mut token).extra();
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
    }
}
