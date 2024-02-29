mod builder;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, GenericParam, Generics};

use builder::fields_builder;

/*

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    #[allow(dead_code)]
    model: [[f32; 4]; 4],
}

impl VertexLayout for InstanceRaw {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
*/

#[proc_macro_derive(VertexLayout, attributes(attributes))]
pub fn vertex_layout(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item);
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = input;

    let attrs = quote! {
        const ATTRIBUTES: &'static [wgpu::VertexAttribute] =
    };

    quote! {
        impl VertexLayout for #ident {

        }
    }
    .into()
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
