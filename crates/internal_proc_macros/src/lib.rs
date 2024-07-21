#![feature(proc_macro_span)]

use proc_macro::TokenStream;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use lazy_static::lazy_static;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::enable_disable::get_enable_disable_field;

mod enable_disable;

#[proc_macro_derive(EnableDisable, attributes(enable_disable))]
pub fn derive_enable_disable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let field = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => get_enable_disable_field(&fields.named),
            Fields::Unnamed(fields) => get_enable_disable_field(&fields.unnamed),
            Fields::Unit => {
                return syn::Error::new(
                    data.fields.span(),
                    "EnableDisable does not support unit structs",
                )
                .into_compile_error()
                .into()
            }
        },
        _ => {
            return syn::Error::new(input.span(), "EnableDisable only supports structs")
                .into_compile_error()
                .into()
        }
    };

    let (field_pos, field) = match field {
        Ok(field) => field,
        Err(err) => return err.into_compile_error().into(),
    };

    let field_name = field
        .ident
        .as_ref()
        .map(|i| quote! { #i })
        .unwrap_or_else(|| quote! { #field_pos });
    let field_type = &field.ty;

    let dummy_impl = quote! {
        const _: fn() = || {
            // this enforces that the fields type implements EnableDisable
            fn assert_impl_enable_disable<T: internal_shared::enable_disable::EnableDisable>() {}
            assert_impl_enable_disable::<#field_type>();
        };
    };

    let expanded = quote! {
        impl internal_shared::enable_disable::GetEnabledDisabled for #name {
            fn is_enabled(&self) -> bool {
                self.#field_name.is_enabled()
            }
        }
        impl internal_shared::enable_disable::SetEnabledDisabled for #name {
            fn set_enabled_disabled(&mut self, enable: bool) {
                self.#field_name.set_enabled_disabled(enable);
            }
        }
        impl internal_shared::enable_disable::EnableDisable for #name {}

        #dummy_impl
    };

    TokenStream::from(expanded)
}

lazy_static! {
    static ref REFLECT_TYPES: Mutex<HashMap<String, HashSet<String>>> = Mutex::new(HashMap::new());
    static ref REFLECT_TYPES_TYPE_BINDER: Mutex<HashMap<String, String>> =
        Mutex::new(HashMap::new());
}

#[proc_macro_derive(AutoRegisterType)]
pub fn auto_register_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let name = ident.to_string();

    let file_path = proc_macro2::Span::call_site()
        .unwrap()
        .source_file()
        .path()
        .display()
        .to_string();

    if let Some(type_binder) = REFLECT_TYPES_TYPE_BINDER
        .lock()
        .unwrap()
        .get(&file_path.clone())
    {
        return syn::Error::new(ident.span(), format!("{ident} deriving AutoRegisterType needs to be above the the types binder deriving RegisterTypeBinder {type_binder}")).into_compile_error().into();
    }

    REFLECT_TYPES
        .lock()
        .unwrap()
        .entry(file_path)
        .or_default()
        .insert(name);

    TokenStream::new()
}

#[proc_macro_derive(RegisterTypeBinder)]
pub fn derive_register_type_binder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let name = ident.to_string();
    let file_path = proc_macro2::Span::call_site()
        .unwrap()
        .source_file()
        .path()
        .display()
        .to_string();

    let types = REFLECT_TYPES.lock().unwrap();
    let types = types.get(&file_path);

    if let Some(type_binder) = REFLECT_TYPES_TYPE_BINDER
        .lock()
        .unwrap()
        .insert(file_path.clone(), name.clone())
    {
        return syn::Error::new(ident.span(), format!("RegisterTypeBinder should not be derived more than once in the same file - derived a second time by {name}, first derived by {type_binder}")).into_compile_error().into();
    }

    let mut register_calls = Vec::new();

    if let Some(types) = types {
        for ty in types {
            let ty_ident = syn::Ident::new(ty, proc_macro2::Span::call_site());
            register_calls.push(quote! {
                app.register_type::<#ty_ident>();
            });
        }
    } else {
        register_calls.push(quote! {
            warn!("RegisterTypeBinder derived in {} where nothing derived AutoRegisterType", #file_path);
        });
    }

    let expanded = quote! {
        impl internal_shared::register_type_binder::RegisterTypeBinder for #ident {
            fn register_types(self, app: &mut App) {
                #(#register_calls)*
            }
        }

        impl #ident {
            pub fn register_types(self, app: &mut App) {
                #(#register_calls)*
            }
        }
    };

    TokenStream::from(expanded)
}
