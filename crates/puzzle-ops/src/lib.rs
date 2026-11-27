use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

#[proc_macro_derive(PuzzleOps)]
pub fn puzzle_ops_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let mut generics = input.generics;
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: ::rubiks_cube::Puzzle));

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let impl_from_ref_alg = quote! {
        impl #impl_generics ::std::convert::From<&::rubiks_cube::Algorithm<Self>> for #name #ty_generics #where_clause {
            fn from(value: &::rubiks_cube::Algorithm<Self>) -> Self {
                ::rubiks_cube::Puzzle::apply(&<Self as ::core::default::Default>::default(), value)
            }
        }
    };
    let impl_from_alg = quote! {
        impl #impl_generics ::std::convert::From<::rubiks_cube::Algorithm<Self>> for #name #ty_generics #where_clause {
            fn from(value: ::rubiks_cube::Algorithm<Self>) -> Self {
                ::rubiks_cube::Puzzle::apply(&<Self as ::core::default::Default>::default(), &value)
            }
        }
    };
    let impl_mul_ref_move = quote! {
        impl #impl_generics ::std::ops::Mul<&<Self as ::rubiks_cube::Puzzle>::Move> for #name #ty_generics #where_clause {
            type Output = Self;
            fn mul(self, rhs: &<Self as ::rubiks_cube::Puzzle>::Move) -> Self::Output {
                self * *rhs
            }
        }
    };
    let impl_mul_ref_alg = quote! {
        impl  #impl_generics ::std::ops::Mul<&::rubiks_cube::Algorithm<Self>> for #name #ty_generics #where_clause {
            type Output = Self;
            fn mul(self, rhs: &::rubiks_cube::Algorithm<Self>) -> Self::Output {
                rhs.into_iter().fold(self, |p, &m| p * m)
            }
        }
    };
    let impl_mul_ref_step = quote! {
        impl #impl_generics ::std::ops::Mul<&::rubiks_cube::StepSolution<Self>> for #name #ty_generics #where_clause {
            type Output = Self;
            fn mul(self, rhs: &::rubiks_cube::StepSolution<Self>) -> Self::Output {
               <Self as ::std::ops::Mul<&::rubiks_cube::Algorithm<Self>>>::mul(self, ::rubiks_cube::StepSolution::moves(rhs))
            }
        }
    };

    let all_impls = quote! {
        #impl_from_ref_alg
        #impl_from_alg
        #impl_mul_ref_move
        #impl_mul_ref_alg
        #impl_mul_ref_step
    };

    all_impls.into()
}
