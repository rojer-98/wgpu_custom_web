mod builder;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, GenericParam, Generics, Lit};

use builder::fields_builder;

#[proc_macro_derive(VertexLayout, attributes(attributes))]
pub fn vertex_layout(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item);
    let DeriveInput { attrs, ident, .. } = input;

    let mut wgpu_attrs: proc_macro2::TokenStream = "".parse().unwrap();
    let mut step_mode: proc_macro2::TokenStream = "".parse().unwrap();
    for attr in &attrs {
        if attr.path().is_ident("attributes") {
            if let Lit::Str(val) = attr.parse_args().unwrap() {
                if val.value().contains("Vertex") || val.value().contains("Instance") {
                    step_mode = val.parse().unwrap();
                } else {
                    wgpu_attrs = val.parse().unwrap();
                }
            }
        }
    }

    quote! {
        impl VertexLayout for #ident {
            const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![#wgpu_attrs];

            fn desc() -> wgpu::VertexBufferLayout<'static> {
                use std::mem::size_of;

                wgpu::VertexBufferLayout {
                    array_stride: size_of::<Self>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::#step_mode,
                    attributes: &Self::ATTRIBUTES,
                }
            }
        }
    }.into()
}

#[proc_macro_derive(Builder)]
pub fn builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let name = ident;
    let generics = add_trait_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields = fields_builder(&data);

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #fields
        }
    }
    .into()
}

// Add a bound `T: HeapSize` to every type parameter T.
pub(crate) fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(heapsize::HeapSize));
        }
    }
    generics
}
