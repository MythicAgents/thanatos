command "cat" "linux" {
  loadtype = "library"
  path = "./target/x86_64-unknown-linux-gnu/release/libcommands_cd.so"

  internal {
    force = true
    detached = false
  }
}

command "pwd" "windows" {
  loadtype = "coff"
  entrypoint = "go"
}
