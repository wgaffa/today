use proc_macro::TokenStream;
use syn::{Data, DataStruct, DeriveInput, Fields, GenericParam, Generics, TypeParamBound, parse_macro_input, parse_quote};
use syn::spanned::Spanned;
use quote::{quote, quote_spanned};

#[proc_macro_derive(Semigroup)]
pub fn semigroup(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = parse_macro_input!(input);

    let generics = add_trait_bounds(generics, parse_quote!(Semigroup));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let combine_fn = match data {
        Data::Struct(DataStruct{ fields: Fields::Named(ref fields), .. }) => {
            let recurse = fields.named.iter().map(|f| {
                let name = &f.ident;
                quote_spanned! {f.span()=>
                    #name: self.#name.combine(rhs.#name)
                }
            });

            quote! {
                fn combine(self, rhs: Self) -> Self {
                    Self {
                        #( #recurse ),*
                    }
                }
            }
        },
        Data::Struct(DataStruct{ fields: Fields::Unnamed(ref fields), .. }) => {
            let i = (0..fields.unnamed.len()).map(syn::Index::from);
            quote! {
                fn combine(self, rhs: Self) -> Self {
                    Self( #( self.#i.combine(rhs.#i) ),* )
                }
            }
        },
        _ => unimplemented!(),
    };

    let output = quote! {
        impl #impl_generics Semigroup for #ident #ty_generics #where_clause {
            #combine_fn
        }
    };

    output.into()
}

#[proc_macro_derive(Monoid)]
pub fn monoid(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = parse_macro_input!(input);

    let generics = add_trait_bounds(generics, parse_quote!(Monoid));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let monoid_fn = match data {
        Data::Struct(DataStruct{ fields: Fields::Named(ref fields), .. }) => {
            let recurse = fields.named.iter().map(|f| {
                let name = &f.ident;
                quote_spanned! {f.span()=>
                    #name: Monoid::empty()
                }
            });

            quote! {
                fn empty() -> Self {
                    Self {
                        #( #recurse ),*
                    }
                }
            }
        },
        Data::Struct(DataStruct{ fields: Fields::Unnamed(ref fields), .. }) => {
            let recurse = fields.unnamed.iter().map(|f| {
                quote_spanned! {f.span()=>
                    Monoid::empty()
                }
            });

            quote! {
                fn empty() -> Self {
                    Self( #( #recurse ),* )
                }
            }
        },
        _ => unimplemented!(),
    };

    let output = quote! {
        impl #impl_generics Monoid for #ident #ty_generics #where_clause {
            #monoid_fn
        }
    };

    output.into()
}

fn add_trait_bounds(mut generics: Generics, ty: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(ty.clone());
        }
    }

    generics
}
