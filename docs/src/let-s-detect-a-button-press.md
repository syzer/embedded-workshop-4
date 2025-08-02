# Let's detect a button press

You can also use your GPIOs as input pins, to detect digital `highs` and `lows`.

Our board has a boot button. After the boot phase, you can use it as an
input GPIO pin. The Boot button is wired to `GPIO9`.

You can detect inputs in several ways:
 - Polling, if the button was pressed
 - Interrupts, if the button was pressed

## Polling

To define an input pin with the esp-hal:

```rust
let config = InputConfig::default().with_pull(Pull::Up);
let button = Input::new(peripherals.GPIO9, config);
```

**Note: What is a Pull-Up?**
If the button is not pressed, the input pin will be pulled high to the 3.3V,
which is the boards high logic level.
If you press the button, the input pin will be pulled low to the 0V,
which is the boards low logic level.

You can now try to implement checking the button state with

```rust
button.is_low()
```

in an endless loop (polling), to check, if the button was pressed.
If you get stuck or need help - `code/polling` is a possible solution.

## The interrupt way

Here we use an interrupt to detect a button press.

```sh
esp-generate --chip esp32c3 --headless -o probe-rs -o defmt interrupt
cd interrupt
cargo add critical-sectiocargo
add esp-hal@=1.0.0-rc.0 -F defmt -F esp32c3 -F unstable
```

If you want to follow along, otherwise the solution is in `code/interrupt`.

**Note: What's an interrupt?**

- When an interrupt is triggered, the CPU will pause its current task and execute the interrupt handler.
- After the interrupt handler is executed, the CPU will resume the previous task.

**Note: Why is a critical section needed?**

- Another interrupt might happend, while we execute our current interrupt.
- It temporary disables interrupts to prevent race conditions.


First we need a dastructure, that is safe to share between interrupts:

```rust
static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None))
```

Then we can define our interrupt handler, which is function and acts like a callback:

```rust
#[handler]
fn my_interrupt() {
    critical_section::with(|cs| {
        info!("Button Pressed");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
```

With this defined callback, we then can setup our interrupt:

```rust
let mut io = Io::new(peripherals.IO_MUX);
io.set_interrupt_handler(my_interrupt);
```

At last, we then need to set the correct pin (`GPIO9` - our boot button) to our button.
This needs to happen in a critical section, so no one else messes
with our setup in between:

```rust
let mut button = Input::new(peripherals.GPIO9, InputConfig::default());
critical_section::with(|cs| {
    button.listen(Event::FallingEdge);
    BUTTON.borrow_ref_mut(cs).replace(button)
});
```

If you flash and run this with

```sh
cargo run --release
```

You should see the something, if you added an `info!(...)` print
to your interrupt function, if you click the boot button.

## TODO: The embassy interrupt way and polling way: Show how much easier this is :)

TODO: do an embassy polling and interrupt example
