use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields, GenericArgument, Index, PathArguments, Type};

pub(crate) fn new_builder(data: &Data) -> TokenStream {
    let fields = match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;

                    quote! {
                        #name: Default::default(),
                    }
                });

                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unnamed(ref fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    let index = Index::from(i);

                    quote! {
                        #index: Default::default(),
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            _ => quote! {},
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    };

    quote! {
        pub fn new() -> Self {
            Self {
                #fields
            }
        }
    }
}

pub(crate) fn fields_builder(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let typ = &f.ty;

                    if let Type::Path(type_path) = typ {
                        let segments = &type_path.path.segments;

                        if let Some(path_segment) = segments.first() {
                            if path_segment.ident.to_string().contains("Option") {
                                if let PathArguments::AngleBracketed(args) = &path_segment.arguments
                                {
                                    let first = args.args.first().unwrap();
                                    if let GenericArgument::Type(sub_typ) = first {
                                        return quote! {
                                            fn #name(mut self, #name: #sub_typ) -> Self {
                                                self.#name = Some(#name);
                                                self
                                            }
                                        };
                                    }
                                }
                            }
                        }
                    }

                    quote! {
                        fn #name(mut self, #name: #typ) -> Self {
                            self.#name = #name;
                            self
                        }
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unnamed(ref fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let typ = &f.ty;
                    let index = Index::from(i);
                    let name = format!("field_{}", index.index);

                    quote! {
                        fn #name(mut self, #name: #typ) -> Self {
                            self.#index = #name;
                            self
                        }
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            _ => quote! {},
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
