pub trait PicoArgsExt {
    fn contains_any<A: Into<pico_args::Keys> + Copy>(self: &mut Self, keys: A) -> bool;
}

impl PicoArgsExt for pico_args::Arguments {
    fn contains_any<A: Into<pico_args::Keys> + Copy>(self: &mut Self, keys: A) -> bool {
        let contains = self.contains(keys);

        // Eat duplicates of the same parameter
        while self.contains(keys) {}

        return contains;
    }
}
