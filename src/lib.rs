use proc_macro::TokenStream;
use syn::{ItemFn, FnArg};
use syn::export::ToTokens;

#[proc_macro_attribute]
pub fn djanco(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let function: ItemFn = syn::parse(item.clone())
        .expect(&format!("Could not parse function: {}", item.to_string()));

    let arguments: Vec<FnArg> = function.sig.inputs.clone().into_iter().collect();

    if arguments.len() != 2 {
        panic!("A function tagged as `djanco` must have 2 arguments, but function `{}` has {}: {:?}",
               function.sig.ident.to_string(),
               function.sig.inputs.len(),
               function.sig.inputs.to_token_stream().to_string());
    }

    // TODO It would be nice to verify the signature here, but
    //      given the amount of information available, it's not
    //      easy to do in the general case, so the options are either
    //      to check with stricter requirements than necessary,
    //      do a warning instead of an error if the check fails, or
    //      forgo checking and let the compiler complain when the runner
    //      bootstrap code is generated.

    // if let Some(FnArg::Typed(argument)) = arguments.get(0) {
    //     println!("db := {:?}", argument);
    //     println!("   ty {:?}", argument.ty);
    //     println!("   attrs {:?}", argument.attrs);
    //     println!("   pat {:?}", argument.pat);
    //     //let pattern: &Option<Ident> = argument.pat;
    //     //let typ: Ty = argument.ty;
    // } else {
    //     panic!("unexpected!");
    // }

    item
}