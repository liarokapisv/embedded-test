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

impl<'c> FlowCollector<Token, Event, Context<'c>> for Handler1 {
    fn emit(&mut self, _token: &mut Token, _value: &Event, context: &mut Context) {
        context.counter += 2;
        *context.x += 1;
    }
}

impl<'c> FlowCollector<Token, Event, Context<'c>> for Handler2 {
    fn emit(&mut self, _token: &mut Token, _value: &Event, context: &mut Context) {
        context.counter -= 1;
        *context.x += 1;
    }
}

#[pin_data]
struct ComponentA<'h, 'c> {
    #[pin]
    handle1: FlowHandle<'h, Token, Event, Context<'c>, Handler1>,
    #[pin]
    handle2: FlowHandle<'h, Token, Event, Context<'c>, Handler2>,
}

impl<'h, 'c> ComponentA<'h, 'c> {
    fn new<'f>(flow: Pin<&'f Flow<'h, Token, Event, Context<'c>>>) -> impl PinInit<Self> + 'f
    where
        'c: 'h,
    {
        pin_init!(Self {
            handle1 <- FlowHandle::new(flow, Handler1()),
            handle2 <- FlowHandle::new(flow, Handler2())
        })
    }
}

#[pin_data]
struct ComponentB<'h, 'c> {
    #[pin]
    comp1: ComponentA<'h, 'c>,
    #[pin]
    comp2: ComponentA<'h, 'c>,
}

impl<'h, 'c> ComponentB<'h, 'c> {
    fn new<'f>(flow: Pin<&'f Flow<'h, Token, Event, Context<'c>>>) -> impl PinInit<Self> + 'f
    where
        'c: 'h,
    {
        pin_init!(Self {
            comp1 <- ComponentA::new(flow),
            comp2 <- ComponentA::new(flow),
        })
    }
}

#[embassy_executor::main]
async fn main(mut _spawner: Spawner) {
    let _p = init(Default::default());

    let mut token = Token::new().expect("only created once!");
    let mut x: u32 = 0;
    let mut context = Context {
        counter: 0,
        x: &mut x,
    };

    stack_pin_init!(let flow = Flow::<Token, Event, Context>::new());
    stack_pin_init!(let component = ComponentB::new(flow.as_ref()));

    loop {
        flow.emit(&mut token, &Event::Event, &mut context);
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        component.comp1.handle1.borrow_mut(&mut token).extra();
        component.comp1.handle2.borrow_mut(&mut token).extra();
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
        component.comp1.handle1.borrow_mut(&mut token).extra();
        component.comp1.handle2.borrow_mut(&mut token).extra();
        defmt::info!("counter = {}", context.counter);
        Timer::after(Duration::from_millis(1000)).await;
    }
}
