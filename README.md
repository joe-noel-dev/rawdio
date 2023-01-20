# Rust Audio Engine

[![Rust](https://github.com/joefocusrite/rawdio/actions/workflows/rust.yml/badge.svg)](https://github.com/joefocusrite/rawdio/actions/workflows/rust.yml)

!! This is a work in progress !!

This is an audio engine, inspired by the Web Audio API.

## Oscillator Example

More examples can be found [here](./examples)

1. Create an engine

    ```rust
    use rawdio::{create_engine, Level, Oscillator, Node};
    let sample_rate = 44_100;
    let (mut context, mut process) = create_engine(sample_rate);
    ```

1. Create an oscillator

    ```rust
    let frequency = 440.0;
    let mut oscillator = Oscillator::new(context.get_command_queue(), frequency);
    ```

1. Set the gain on the oscillator

    ```rust
    let gain = Level::from_db(-3.0);
    oscillator
        .gain
        .set_value_at_time(gain.as_gain(), Timestamp::zero());
    ```

1. Connect to output

    ```rust
    oscillator.connect_to_output();
    ```

1. Start the context

    ```rust
    context.start();
    ```

1. Run the process to get samples. This will vary depending on whether
   you wish to run the engine in realtime (e.g. using CPAL) or offline (e.g. to a file).
   The engine doesn't make any assumptions, and will simply wait to be asked to process.

   All audio buffers are assumed to be non-interleaved.
   So if the audio that comes from your soundcard is interleaved, it will need
   to be de-interleaved first.

    ```rust
    let output_buffer = /*create an audio buffer (see `OwnedAudioBuffer`) or use the one supplied by your audio device in its callback*/
    process.process (&mut output_buffer);
    ```

## To run an example

```sh
cargo run --example [example_name] [example_args]
```

## To run the tests

```sh
cargo test
```

## To run the benchmarks

```sh
cargo bench
```
