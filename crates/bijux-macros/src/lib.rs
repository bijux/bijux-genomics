use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn fastq_v1_invariant(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
