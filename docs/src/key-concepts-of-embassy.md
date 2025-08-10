# Key Concepts of embassy

## What is embassy

_Embassy is the next-generation framework for embedded applications. Write safe, correct, and energy-efficient embedded code faster, using the Rust programming language, its async facilities, and the Embassy libraries._
<sub>[Source](https://github.com/embassy-rs/embassy?tab=readme-ov-file#embassy)</sub>

TL;DR: It brings Multitasking to the embedded world without an OS on bare metal

## Key concepts of embassy

- **An executor**: A runtime scheduling and running async tasks
- **Tasks**: Async functions, that yield control when waiting for I/O or timers.
- Everything is bare-metal without any heap - tasks are allocated at compile time.

Let's have a look at `code/hello_embassy`

### The main entry point

The main entry point for an embassy project is its async main.

```rust
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    info!("Embassy initialized!");

    spawner
        .spawn(my_led_blink_task())
        .expect("Could not spawn LED task");
    spawner
        .spawn(my_other_things())
        .expect("Could not spawn my other task")
}
```

Changes you see from _normal_ bare metal:

- Other macro on top
- No infinite loop in main
- You **spawn** tasks in this *main*

## The actual work

The tasks then usually carry out the jobs - asynchronous. They are marked async as well and are attributed
by the `#[embassy_executor::task]` macro.

```rust
#[embassy_executor::task]
async fn my_led_blink_task() {
    loop {
        info!("On");
        Timer::after_millis(500).await;
        info!("Off");
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn my_other_things() {
    loop {
        info!("Other GPIO HIGH");
        Timer::after_millis(1000).await;
        info!("Other GPIO LOW");
        Timer::after_millis(1000).await;
    }
}
```

## Communication between tasks

Let's have a look at `code/embassy_polling_button`.

**What is the code doing?**

One task polls the state of the your boot button (`GPIO9`).
If the button state changes, it sends an event. The other task,
waiting for events, will wake up and process the incoming event.

To send such events, we need a so called `CHANNEL` (global). It's a queue, where
we can enqueue data, and in this case, can hold up to 10 elements in the queue.

```rust
static BUTTON_CHANNEL: Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    ButtonEvent,
    10,
> = Channel::new();

#[derive(Clone, Copy, defmt::Format)]
enum ButtonEvent {
    Pressed,
    Released,
}
```

The sending task (polling button state) gets the sender part of the channel

```rust
let sender = BUTTON_CHANNEL.sender();
```

And the other task the receiving end:

```rust
let receiver = BUTTON_CHANNEL.receiver();
```

The cool thing! The receiver can sleep, until it receives it and event from the queue:

```rust
info!("I am idle and waiting for an event");
let event = receiver.receive().await;
```

The executor runtime will continue the computation of that task, when an event is received.

You can try it out.

Go to `code/embassy_polling_task` and run

```sh
cargo run --release
```

## Interrupts in Embassy

TODO: Interrupts in [embassy](https://embassy.dev/book/#_async_version)
