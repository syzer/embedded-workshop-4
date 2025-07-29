# Let's detect a button press

## The lazy way

You can also use your GPIOs as input pins, to detect digital `highs` and `lows`.

Your task is now to use your custom built "button" (A wire with a resistor - or a resistor only).
Therefore you need to configure one of your GPIOs as input pins.

If this GPIO pins detects a connection with higher voltage (connect your button to you 3.3 Volts source), you can `info!(...)` a message, that the button was pressed.

## The interrupt way

TODO: A button press with an interrupt: [from here](https://docs.espressif.com/projects/rust/no_std-training/03_4_interrupt.html)
