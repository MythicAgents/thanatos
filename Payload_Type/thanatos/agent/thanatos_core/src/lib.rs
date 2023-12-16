pub fn entrypoint() {
    let c = config::raw();
    utils::hexdump(c);
}
