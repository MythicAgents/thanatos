use proc_macro::TokenStream;
use syn::parse_quote;

#[cfg(all(target_os = "linux", feature = "std"))]
#[proc_macro_attribute]
pub fn entrypoint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    use syn::{
        Ident, ItemFn,
        __private::{Span, ToTokens},
    };

    let entry: ItemFn = syn::parse(item.clone()).unwrap();
    let entry_fn = entry.sig.ident;
    let init_fn = Ident::new(&format!("__{}_internal", entry_fn), Span::call_site());

    let init_stub: syn::Stmt = parse_quote!(
        #[used]
        #[cfg_attr(target_os = "linux", link_section = ".init_array")]
        pub static INIT: extern "C" fn() = #init_fn;
    );

    let entry_stub: syn::Stmt = parse_quote! {
        #[no_mangle]
        extern "C" fn #init_fn() {
            let internal = match std::env::var("LD_INTERNAL").map(|v| v.parse::<u32>()) {
                Ok(Ok(v)) => v,
                _ => {
                    return;
                }
            };

            std::env::remove_var("LD_INTERNAL");
            let parameters = std::env::var("LD_PARAMS").ok();

            let send_pipe = match std::env::var("LD_SENDPIPE") {
                Ok(p) => p,
                _ => if internal == 1 {
                        return;
                    } else {
                        std::process::exit(0);
                    }
            };

            // Command is loaded internally
            if internal == 1 {
                #entry_fn(parameters);
            } else {
                std::env::remove_var("LD_PRELOAD");
                #entry_fn(parameters);
                std::process::exit(0);
            }
        }
    };

    let init_tok: TokenStream = init_stub.into_token_stream().into();
    let entry_tok: TokenStream = entry_stub.into_token_stream().into();

    let mut final_tok = TokenStream::new();
    final_tok.extend(init_tok);
    final_tok.extend(entry_tok);
    final_tok.extend(item);
    final_tok
}
