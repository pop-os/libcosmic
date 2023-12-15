use proc_macro::TokenStream;
use quote::quote;
use syn::{self};

#[proc_macro_derive(CosmicConfigEntry, attributes(version, id))]
pub fn cosmic_config_entry_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_cosmic_config_entry_macro(&ast)
}

fn impl_cosmic_config_entry_macro(ast: &syn::DeriveInput) -> TokenStream {
    let attributes = &ast.attrs;
    let version = attributes
        .iter()
        .find_map(|attr| {
            if attr.path.is_ident("version") {
                match attr.parse_meta() {
                    Ok(syn::Meta::NameValue(syn::MetaNameValue {
                        lit: syn::Lit::Int(lit_int),
                        ..
                    })) => Some(lit_int.base10_parse::<u64>().unwrap()),
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap_or(0);

    let name = &ast.ident;

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
            cosmic_config::ConfigSet::set(config, stringify!(#field_name), &self.#field_name)?;
        }
    });

    let get_each_config_field = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            match cosmic_config::ConfigGet::get::<#field_type>(config, stringify!(#field_name)) {
                Ok(#field_name) => default.#field_name = #field_name,
                Err(e) => errors.push(e),
            }
        }
    });

    let update_each_config_field = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            stringify!(#field_name) => {
                match cosmic_config::ConfigGet::get::<#field_type>(config, stringify!(#field_name)) {
                    Ok(value) => {
                        if self.#field_name != value {
                            keys.push(stringify!(#field_name));
                        }
                        self.#field_name = value;
                    },
                    Err(e) => {
                        errors.push(e);
                    }
                }
            }
        }
    });

    let gen = quote! {
        impl CosmicConfigEntry for #name {
            const VERSION: u64 = #version;

            fn write_entry(&self, config: &cosmic_config::Config) -> Result<(), cosmic_config::Error> {
                let tx = config.transaction();
                #(#write_each_config_field)*
                tx.commit()
            }

            fn get_entry(config: &cosmic_config::Config) -> Result<Self, (Vec<cosmic_config::Error>, Self)> {
                let mut default = Self::default();
                let mut errors = Vec::new();

                #(#get_each_config_field)*

                if errors.is_empty() {
                    Ok(default)
                } else {
                    Err((errors, default))
                }
            }

            fn update_keys<T: AsRef<str>>(&mut self, config: &cosmic_config::Config, changed_keys: &[T]) -> (Vec<cosmic_config::Error>, Vec<&'static str>){
                let mut keys = Vec::with_capacity(changed_keys.len());
                let mut errors = Vec::new();
                for key in changed_keys.iter() {
                    match key.as_ref() {
                        #(#update_each_config_field)*
                        _ => (),
                    }
                }
                (errors, keys)
            }
        }
    };

    gen.into()
}
