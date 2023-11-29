command "pwd" "linux" {
  loadtype = "library"
  path = "./libcommands_pwd.so"

  # Command should not be ran in a background thread
  background = false

  # Command should only be loaded internally regardless of execution method
  internal_only = true
}

command "pwd" "windows" {
  loadtype = "coff"
  entrypoint = "go"

  background = false

  internal_only = true
}
