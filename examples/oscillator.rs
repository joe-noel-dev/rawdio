use rust_audio_engine::context::Context;

fn main() {
    let context = Context::new(44100);
    println!("Current time = {}", context.current_time().get_seconds());
}
