use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Rerankable, attributes(rerank_field))]
pub fn derive_rerankable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Rerankable can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Rerankable can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let marked_fields: Vec<_> = fields
        .iter()
        .filter(|f| {
            f.attrs
                .iter()
                .any(|attr| attr.path().is_ident("rerank_field"))
        })
        .collect();

    if marked_fields.is_empty() {
        return syn::Error::new_spanned(
            &input,
            "exactly one field must be marked with #[rerank_field]",
        )
        .to_compile_error()
        .into();
    }

    if marked_fields.len() > 1 {
        return syn::Error::new_spanned(
            &marked_fields[1].ident,
            "only one field can be marked with #[rerank_field]",
        )
        .to_compile_error()
        .into();
    }

    let field = marked_fields[0];
    let field_name = field.ident.as_ref().unwrap().to_string();

    let is_string = match &field.ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "String")
            .unwrap_or(false),
        _ => false,
    };

    if !is_string {
        return syn::Error::new_spanned(
            &field.ty,
            "#[rerank_field] can only be applied to a field of type String",
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {
        impl umem_rerank::Rerankable for #name {
            const RANK_FIELD: &'static str = #field_name;
        }
    };

    TokenStream::from(expanded)
}
