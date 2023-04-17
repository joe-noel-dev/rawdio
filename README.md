# Rust Audio Engine

[![Rust](https://github.com/joefocusrite/rawdio/actions/workflows/rust.yml/badge.svg)](https://github.com/joefocusrite/rawdio/actions/workflows/rust.yml)

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
    let output_channel_count = 2;
    let mut oscillator = Oscillator::sine(context.as_ref(), frequency, output_channel_count);
    ```

1. Set the gain on the oscillator

    ```rust
    let level = Level::from_db(-3.0);
    oscillator
        .gain
        .set_value_at_time(level.as_gain(), Timestamp::zero());
    ```

1. Connect to output

    ```rust
    oscillator.node.connect_to_output();
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
    let input_buffer = /*create an input buffer*/
    let output_buffer = /*create an audio buffer*/
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

## Where do the buffers come from?

The engine won't make any assumptions about how it is going to be run.
This means that it can be run in real-time, for example using CPAL.
Or, it could be run offline, for example processing audio from files using hound.
There are examples of both of these in the [/examples](examples) directory.

Bear in mind that audio is expected to be de-interleaved.
Most soundcards and audio files will be interleaved, so it will need to be converted first.
