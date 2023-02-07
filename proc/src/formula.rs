use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::attrs::parse_attributes;

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let ident = &input.ident;

    let args = parse_attributes(&input.attrs)?;
    let non_exhaustive = args.non_exhaustive.is_some();

    if let Some(formula) = args
        .serialize
        .or(args.deserialize)
        .or(args.common)
        .or(args.owned.flatten())
    {
        return Err(syn::Error::new_spanned(
            formula.path,
            "Formula type should not be specified for `Serialize` and `Deserialize` when type is also `Formula`",
        ));
    }

    if args.variant.is_some() {
        return Err(syn::Error::new_spanned(
            input,
            "Variant should not be specified for `Serialize` when type is also `Formula`",
        ));
    }

    match &input.data {
        syn::Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Formula cannot be derived for unions",
            ))
        }
        syn::Data::Struct(data) => {
            let all_field_types: Vec<_> = data.fields.iter().map(|field| &field.ty).collect();

            let mut formula_generics = input.generics.clone();
            if !all_field_types.is_empty() && !input.generics.params.is_empty() {
                let predicates = all_field_types.iter().map(|ty| -> syn::WherePredicate {
                    syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::private::Formula }
                });

                let where_clause = formula_generics.make_where_clause();
                where_clause.predicates.extend(predicates);
            }

            let field_count = data.fields.len();

            let field_check_names = match &data.fields {
                syn::Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|field| {
                        quote::format_ident!(
                            "__ALKAHEST_FORMULA_FIELD_{}_IDX",
                            field.ident.as_ref().unwrap(),
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            };

            let field_check_ids = match &data.fields {
                syn::Fields::Named(fields) => (0..fields.named.len()).collect(),
                _ => Vec::new(),
            };

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (formula_impl_generics, formula_type_generics, formula_where_clause) =
                formula_generics.split_for_impl();

            let expand_size = if non_exhaustive {
                quote::quote! {
                    max_size = ::alkahest::private::Option::None;
                }
            } else {
                quote::quote! {}
            };

            Ok(quote::quote! {
                impl #impl_generics #ident #type_generics #where_clause {
                    #(
                        #[doc(hidden)]
                        #[inline(always)]
                        #[allow(non_upper_case_globals)]
                        pub const #field_check_names: [(); #field_check_ids] = [(); #field_check_ids];
                    )*

                    #[doc(hidden)]
                    #[inline(always)]
                    pub const __ALKAHEST_FORMULA_FIELD_COUNT: [(); #field_count] = [(); #field_count];
                }

                impl #formula_impl_generics ::alkahest::private::NonRefFormula for #ident #formula_type_generics #formula_where_clause {
                    const MAX_SIZE: ::alkahest::private::Option<::alkahest::private::usize> = {
                        #[allow(unused_mut)]
                        let mut max_size = Some(0);
                        #(
                            max_size = ::alkahest::private::sum_size(max_size, <#all_field_types as ::alkahest::private::Formula>::MAX_SIZE);
                        )*;
                        #expand_size
                        max_size
                    };
                }
            })
        }
        syn::Data::Enum(data) => {
            let all_field_types: Vec<Vec<&syn::Type>> = data
                .variants
                .iter()
                .map(|variant| variant.fields.iter().map(|field| &field.ty).collect())
                .collect();

            let all_field_types_flat: Vec<&syn::Type> = data
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| &field.ty))
                .collect();

            let mut formula_generics = input.generics.clone();
            if !all_field_types_flat.is_empty() && !input.generics.params.is_empty() {
                let predicates = all_field_types_flat
                    .iter()
                    .map(|ty| -> syn::WherePredicate {
                        syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::private::Formula }
                    });

                let where_clause = formula_generics.make_where_clause();
                where_clause.predicates.extend(predicates);
            }

            let field_check_names: Vec<Vec<syn::Ident>> = data
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    syn::Fields::Named(fields) => fields
                        .named
                        .iter()
                        .map(|field| {
                            quote::format_ident!(
                                "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_{}_IDX",
                                variant.ident,
                                field.ident.as_ref().unwrap(),
                            )
                        })
                        .collect(),
                    _ => Vec::new(),
                })
                .collect();

            let field_check_ids: Vec<Vec<usize>> = data
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    syn::Fields::Named(fields) => (0..fields.named.len()).collect(),
                    _ => Vec::new(),
                })
                .collect();

            let field_count: Vec<usize> = data
                .variants
                .iter()
                .map(|variant| variant.fields.len())
                .collect();

            let field_variant_name_ids: Vec<syn::Ident> = data
                .variants
                .iter()
                .map(|variant| {
                    quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_IDX", variant.ident,)
                })
                .collect();

            let field_variant_ids: Vec<_> = (0..data.variants.len() as u32).collect();

            let field_count_checks: Vec<syn::Ident> =
                data.variants
                    .iter()
                    .map(|variant| {
                        quote::format_ident!(
                            "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_COUNT",
                            variant.ident,
                        )
                    })
                    .collect();

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (formula_impl_generics, formula_type_generics, formula_where_clause) =
                formula_generics.split_for_impl();

            let expand_size = if non_exhaustive {
                quote::quote! {
                    max_size = ::alkahest::private::Option::None;
                }
            } else {
                quote::quote! {}
            };

            Ok(quote::quote! {
                impl #impl_generics #ident #type_generics #where_clause {
                    #(#(
                        #[doc(hidden)]
                        #[inline(always)]
                        #[allow(non_upper_case_globals)]
                        pub const #field_check_names: [(); #field_check_ids] = [(); #field_check_ids];
                    )*)*

                    #(
                        #[doc(hidden)]
                        #[inline(always)]
                        #[allow(non_upper_case_globals)]
                        pub const #field_count_checks: [(); #field_count] = [(); #field_count];
                    )*

                    #(
                        #[doc(hidden)]
                        #[inline(always)]
                        #[allow(non_upper_case_globals)]
                        pub const #field_variant_name_ids: u32 = #field_variant_ids;
                    )*
                }

                impl #formula_impl_generics ::alkahest::private::NonRefFormula for #ident #formula_type_generics #formula_where_clause {
                    const MAX_SIZE: ::alkahest::private::Option<::alkahest::private::usize> = {
                        #[allow(unused_mut)]
                        let mut max_size = Some(0);

                        #(
                            let var_size = {
                                #[allow(unused_mut)]
                                let mut max_size = Some(0);
                                #(
                                    max_size = ::alkahest::private::sum_size(max_size, <#all_field_types as ::alkahest::private::Formula>::MAX_SIZE);
                                )*;
                                max_size
                            };
                            max_size = ::alkahest::private::max_size(max_size, var_size);
                        )*

                        #expand_size
                        max_size
                    };
                }
            })
        }
    }
}
