use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_quote};

#[proc_macro_derive(CosmicConfigEntry)]
pub fn cosmic_config_entry_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_cosmic_config_entry_macro(&ast)
}

fn impl_cosmic_config_entry_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    // Get the fields of the struct
    let fields = match ast.data {
        syn::Data::Struct(ref data_struct) => match data_struct.fields {
            syn::Fields::Named(ref fields) => &fields.named,
            _ => unimplemented!("Only named fields are supported"),
        },
        _ => unimplemented!("Only structs are supported"),
    };

    let write_each_config_field = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            config.set(stringify!(#field_name), &self.#field_name)?;
        }
    });

    let get_each_config_field = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            match config.get::<#field_type>(stringify!(#field_name)) {
                Ok(#field_name) => default.#field_name = #field_name,
                Err(e) => errors.push(e),
            }
        }
    });

    // Get the existing where clause or create a new one if it doesn't exist
    let mut where_clause = ast
        .generics
        .where_clause
        .clone()
        .unwrap_or_else(|| parse_quote!(where));

    // Add your additional constraints to the where clause
    // Here, we add the constraint 'T: Debug' to all generic parameters
    for param in ast.generics.params.iter() {
        where_clause
            .predicates
            .push(parse_quote!(#param: ::std::default::Default + ::serde::Serialize + ::serde::de::DeserializeOwned));
    }

    let gen = quote! {
        impl #generics CosmicConfigEntry for #name #generics #where_clause {
            fn write_entry(&self, config: &Config) -> Result<(), cosmic_config::Error> {
                #(#write_each_config_field)*
                Ok(())
            }

            fn get_entry(config: &Config) -> Result<Self, (Vec<cosmic_config::Error>, Self)> {
                let mut default = Self::default();
                let mut errors = Vec::new();

                #(#get_each_config_field)*

                if errors.is_empty() {
                    Ok(default)
                } else {
                    Err((errors, default))
                }
            }
        }
    };

    gen.into()
}
