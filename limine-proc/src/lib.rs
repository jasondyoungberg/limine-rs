extern crate proc_macro;

use proc_macro::TokenStream;

/// Adds the limine request to the `.limine_reqs` section.
///
/// ## Note
/// If the executable kernel file contains a `.limine_reqs` section, the bootloader
/// will, instead of scanning the executable for requests, fetch the requests from
/// a NULL-terminated array of pointers to the provided requests, contained inside
/// said section.
#[proc_macro_attribute]
pub fn limine_tag(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemStatic);

    let typ = input.ty.clone();
    let name = input.ident.clone();
    let ptr_name = quote::format_ident!("{name}_PTR");

    quote::quote! {
        #input

        const _: () = {
            // Limine expects a NULL-terminated array of pointers to the provided requests.
            #[used]
            #[link_section = ".limine_reqs"]
            static #ptr_name: &#typ = &#name;
        };
    }
    .into()
}
