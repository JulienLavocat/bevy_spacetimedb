use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

/// This macro automatically generates the boilerplate code needed to register a reducer
/// with the `StdbPlugin`.
///
/// ## Requirements
///
/// - The struct must have exactly one field named `event` of type `ReducerEvent<Reducer>`
/// - All other fields must match the reducer's parameter types and order
/// - Struct fields must be named (no tuple structs)
///
/// ## Example
///
///```no-run
/// #[derive(RegisterReducerEvent)]
/// pub struct SetName {
///     pub event: ReducerEvent<Reducer>,
///     pub name: String,
/// }
/// ```
#[proc_macro_derive(RegisterReducerEvent)]
pub fn register_reducer_event_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Derive callback name directly from struct name (no suffix stripping)
    let function_name = Ident::new(
        &format!("on_{}", struct_name_str.to_snake_case()),
        struct_name.span(),
    );

    // Extract named fields
    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => panic!("Struct must have named fields"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Separate 'event' field from reducer parameters
    let mut event_field = None;
    let mut param_fields = Vec::new();
    let mut param_idents = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().expect("Field must have identifier");
        if field_ident == "event" {
            if event_field.is_some() {
                panic!("Duplicate 'event' field");
            }
            event_field = Some(field);
        } else {
            param_idents.push(field_ident.clone());
            param_fields.push(field);
        }
    }

    if event_field.is_none() {
        panic!("Struct must have an 'event' field");
    }

    // Generate the implementation
    let expanded = quote! {
        impl bevy_spacetimedb::RegisterableReducerEvent<DbConnection, RemoteModule> for #struct_name {
            fn set_stdb_callback(reducers: &RemoteReducers, sender: std::sync::mpsc::Sender<ReducerResultEvent<Self>>) {
                reducers.#function_name(move |ctx, #(#param_idents),*| {
                    sender
                        .send(ReducerResultEvent::new(#struct_name {
                            event: ctx.event.clone(),
                            #(#param_idents: #param_idents.clone()),*
                        }))
                        .unwrap();
                });
            }
        }
    };

    TokenStream::from(expanded)
}
