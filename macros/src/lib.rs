use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, Result, Token, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct TableEntry {
    table: Ident,
    has_update: bool,
}

impl Parse for TableEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        // Identity with no flags
        if input.peek(Ident) {
            return Ok(TableEntry {
                table: input.parse()?,
                has_update: false,
            });
        }

        // Parse an identity with flags
        let content;
        parenthesized!(content in input);

        let table: Ident = content.parse()?;

        let has_update = if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
            let flag: Ident = content.parse()?;
            if flag != "no_update" {
                return Err(syn::Error::new_spanned(flag, "Expected `no_update`"));
            }

            false
        } else {
            true
        };

        Ok(TableEntry { table, has_update })
    }
}

struct TablesInput {
    entries: Vec<TableEntry>,
}

impl Parse for TablesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut entries = Vec::new();
        while !input.is_empty() {
            entries.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(TablesInput { entries })
    }
}

#[proc_macro]
pub fn tables(input: TokenStream) -> TokenStream {
    let TablesInput { entries } = parse_macro_input!(input as TablesInput);

    let mut output = quote! {};

    for entry in entries {
        let table = &entry.table;
        output.extend(quote! {
            plugin.on_insert(app, db.#table());
            plugin.on_delete(app, db.#table());
        });

        if entry.has_update {
            output.extend(quote! {
                plugin.on_update(app, db.#table());
            });
        }
    }

    output.into()
}

struct ReducerEntry {
    handler_name: Ident,
    params: Punctuated<Ident, Token![,]>,
    struct_expr: Expr,
}

impl Parse for ReducerEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let handler_name: Ident = input.parse()?;
        let content;
        parenthesized!(content in input);
        let params = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;

        input.parse::<Token![=>]>()?;

        let struct_expr: Expr = input.parse()?;

        Ok(ReducerEntry {
            handler_name,
            params,
            struct_expr,
        })
    }
}

struct ReducersInput {
    entries: Vec<ReducerEntry>,
}

impl Parse for ReducersInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut entries = Vec::new();
        while !input.is_empty() {
            entries.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { entries })
    }
}

#[proc_macro]
pub fn register_reducers(input: TokenStream) -> TokenStream {
    let ReducersInput { entries } = parse_macro_input!(input as ReducersInput);

    let mut output = quote! {};

    for entry in entries {
        let handler_name = &entry.handler_name;
        let param_list = &entry.params;
        let struct_expr = &entry.struct_expr;
        let struct_name = if let Expr::Struct(s) = struct_expr {
            &s.path.segments.last().unwrap().ident
        } else {
            panic!("Expected a struct expression");
        };
        let send_ident = Ident::new(
            &format!("send_{}", struct_name.to_string().to_lowercase()),
            struct_name.span(),
        );

        output.extend(quote! {
        let #send_ident = plugin.reducer_event::<#struct_name>(app);
        reducers.#handler_name(move |#param_list| {
            #send_ident
                .send(ReducerResultEvent::new(#struct_expr))
                .unwrap();
        });
        });
    }

    output.into()
}
