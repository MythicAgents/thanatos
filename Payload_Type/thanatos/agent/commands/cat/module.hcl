command "cat" "linux" {
  loadtype = "library"
  path = "./libcommands_cat.so"

  background = false
  internal = true
}

command "cat" "windows" {
  loadtype = "coff"
  entrypoint = "go"
}
