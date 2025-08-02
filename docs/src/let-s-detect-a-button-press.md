# Let's detect a button press

You can also use your GPIOs as input pins, to detect digital `highs` and `lows`.

Our board has a boot button. After the boot phase, you can use it as an
input GPIO pin. The Boot button is wired to `GPIO9`.

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

You can detect inputs in several ways:
 - Polling, if the button was pressed
 - Interrupts, if the button was pressed

## Polling

You can now try to implement checking the button state with

```rust
button.is_low()
```

in an endless loop (polling), to check, if the button was pressed.
If you get stuck or need help - `code/polling` is a possible solution.

## The interrupt way

TODO: A button press with an interrupt: [from here](https://docs.espressif.com/projects/rust/no_std-training/03_4_interrupt.html)

## The embassy interrupt way

TODO: do an embassy polling and interrupt example
