use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Field, FieldsNamed, Ident, Type};

fn is_fieldset(field: Field) -> bool {
    field
        .attrs
        .iter()
        .filter_map(|a| a.path().get_ident())
        .any(|i| *i == format_ident!("fieldset"))
}

fn get_type_identifier(ty: Type) -> Ident {
    match ty {
        Type::Path(p) => {
            assert!(p.clone().qself.is_none());
            p.path
                .get_ident()
                .expect("field type must be a path with an identifier")
                .clone()
        }
        _ => panic!("unsupported field type"),
    }
}

fn get_field_identifier(field: Field) -> Ident {
    field
        .ident
        .expect("Cannot derive field type from tuple structs")
}

fn derive_field_type(name: String, fields: FieldsNamed) -> TokenStream {
    let derived_field_type_identifier = format_ident!("{}FieldType", name);
    let enum_variants = {
        let mut res = Vec::new();
        for field in fields.named {
            let variant_name = format_ident!(
                "{}",
                get_field_identifier(field.clone())
                    .to_string()
                    .to_upper_camel_case()
            );
            if is_fieldset(field.clone()) {
                let type_identifier = get_type_identifier(field.ty);
                let field_type_identifier = format_ident!("{}FieldType", type_identifier);
                res.push(quote!(#variant_name(#field_type_identifier)));
            } else {
                let ty = field.ty;
                res.push(quote!(#variant_name(#ty)));
            }
        }
        res
    };
    quote!(
        pub enum #derived_field_type_identifier {
            #(#enum_variants ,)*
        }
    )
    .into()
}

fn derive_apply_method(name: String, fields: FieldsNamed) -> TokenStream {
    let identifier = format_ident!("{}", name);
    let field_type_identifier = format_ident!("{}FieldType", name);
    let match_arms = {
        let mut res = Vec::new();
        for field in fields.named {
            let field_identifier = get_field_identifier(field.clone());
            let variant_name = format_ident!(
                "{}",
                field_identifier.clone().to_string().to_upper_camel_case()
            );
            if is_fieldset(field.clone()) {
                res.push(quote!(#field_type_identifier::#variant_name(x) => self.#field_identifier.apply(x)));
            } else {
                res.push(
                    quote!(#field_type_identifier::#variant_name(x) => self.#field_identifier = x),
                );
            }
        }
        res
    };
    quote!(
        impl #identifier {
            pub fn apply(&mut self, field: #field_type_identifier) {
                match field {
                    #( #match_arms ,)*
                }
            }
        }
    )
    .into()
}

fn derive_setter_trait(name: String, fields: FieldsNamed) -> TokenStream {
    let derived_setter_trait_identifier = format_ident!("{}Setter", name);
    let methods = {
        let mut res = Vec::new();
        for field in fields.named {
            let method_name = get_field_identifier(field.clone());
            if is_fieldset(field.clone()) {
                let type_identifier = get_type_identifier(field.ty);
                let field_setter_trait_identifier = format_ident!("{}Setter", type_identifier);
                res.push(quote!(fn #method_name(&mut self) -> impl #field_setter_trait_identifier));
            } else {
                let ty = field.ty;
                res.push(quote!(fn #method_name(&mut self) -> impl FieldSetter<#ty>));
            }
        }
        res
    };

    quote!(
        pub trait #derived_setter_trait_identifier {
            #( #methods ;)*
        }
    )
    .into()
}

fn derive_fieldset_type(name: String, fields: FieldsNamed) -> TokenStream {
    let derived_fieldset_identifier = format_ident!("{}FieldSet", name);
    let opt_fields = {
        let mut res = Vec::new();
        for field in fields.named {
            let field_identifier = get_field_identifier(field.clone());
            if is_fieldset(field.clone()) {
                let type_identifier = get_type_identifier(field.ty);
                let fieldset_identifier = format_ident!("{}FieldSet", type_identifier);
                res.push(quote!(#field_identifier : #fieldset_identifier));
            } else {
                let ty = field.ty;
                res.push(quote!(#field_identifier : Option<#ty>))
            }
        }
        res
    };
    quote!(
        pub struct #derived_fieldset_identifier {
            #(#opt_fields ,)*
        }
    )
    .into()
}

fn derive_fieldset_setter_trait_impl(name: String, fields: FieldsNamed) -> TokenStream {
    let setter_trait_identifier = format_ident!("{}Setter", name);
    let fieldset_identifier = format_ident!("{}FieldSet", name);
    let methods = {
        let mut res = Vec::new();
        for field in fields.named {
            let field_name = get_field_identifier(field.clone());
            let method_name = field_name.clone();
            if is_fieldset(field.clone()) {
                let type_identifier = get_type_identifier(field.ty);
                let field_setter_trait_identifier = format_ident!("{}Setter", type_identifier);
                res.push(quote!(fn #method_name(&mut self) -> impl #field_setter_trait_identifier { &mut self.#field_name }));
            } else {
                let ty = field.ty;
                res.push(
                    quote!(fn #method_name(&mut self) -> impl FieldSetter<#ty> { &mut self.#field_name }),
                );
            }
        }
        res
    };

    quote!(
        impl #setter_trait_identifier for &mut #fieldset_identifier {
            #( #methods )*
        }
    )
    .into()
}

fn derive_fieldset_into_iterator(name: String, fields: FieldsNamed) -> TokenStream {
    let fieldset_identifier = format_ident!("{}FieldSet", name);
    let fieldtype_identifier = format_ident!("{}FieldType", name);
    let iter_chains = {
        let mut res = Vec::new();
        for field in fields.named {
            let field_identifier = get_field_identifier(field.clone());
            let variant_name = format_ident!(
                "{}",
                field_identifier.clone().to_string().to_upper_camel_case()
            );
            if is_fieldset(field.clone()) {
                res.push(quote!(let iter = iter.chain(self.#field_identifier.opt_iter().map(|x| x.map(#fieldtype_identifier::#variant_name)))));
            } else {
                res.push(quote!(let iter = iter.chain(once(self.#field_identifier.map(#fieldtype_identifier::#variant_name)))));
            }
        }
        res
    };
    quote!(
        impl #fieldset_identifier {
            fn opt_iter(self) -> impl Iterator<Item = Option<#fieldtype_identifier>> {
                use core::iter::empty;
                use core::iter::once;
                let iter = empty();

                #( #iter_chains ;)*

                iter
            }
        }

        impl IntoIterator for #fieldset_identifier {
            type Item = #fieldtype_identifier;
            type IntoIter = impl Iterator<Item = Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.opt_iter().flatten()
            }
        }
    )
    .into()
}

#[proc_macro_derive(FieldSet, attributes(fieldset))]
pub fn derive_fieldset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if let syn::Data::Struct(ref data) = input.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            let mut result = TokenStream::default();
            result.extend(derive_field_type(input.ident.to_string(), fields.clone()));
            result.extend(derive_apply_method(input.ident.to_string(), fields.clone()));
            result.extend(derive_setter_trait(input.ident.to_string(), fields.clone()));
            result.extend(derive_fieldset_type(
                input.ident.to_string(),
                fields.clone(),
            ));
            result.extend(derive_fieldset_setter_trait_impl(
                input.ident.to_string(),
                fields.clone(),
            ));
            result.extend(derive_fieldset_into_iterator(
                input.ident.to_string(),
                fields.clone(),
            ));
            return result;
        }
    }

    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `FieldEvents`",
        )
        .to_compile_error(),
    )
}
