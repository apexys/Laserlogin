#![recursion_limit="1024"]

use std::str::FromStr;

use proc_macro::TokenStream;
use syn::{DeriveInput, Data, Fields};
use quote::quote;


#[proc_macro_derive(SqlObject)]
pub fn sqlobject_derive(input: TokenStream) -> TokenStream {
     // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let table_name = format!("{}", name);

    //A new list for the field names
    let mut field_names = Vec::new();

    match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    for field in fields.named.iter(){
                        //Run over all fields, push name and type
                        let ident = field.ident.as_ref().unwrap();
                        if *ident != "id"{ //Do not push id field. We could in future use an annotation to designate an id field
                            field_names.push((ident.clone(), field.ty.clone()));
                        }
                    }
                },
                Fields::Unnamed(_) => {panic!("Derivation of SqlObject for Unnamed Struct Fields is not defined")},
                Fields::Unit => {panic!("Derivation of SqlObject for Unit Fields is not defined")}
            }
        }
        Data::Enum(_) => {panic!("Derivation of SqlObject for Enums is not defined")}
        Data::Union(_) => {panic!("Derivation of SqlObject for Unions is not defined")}
    }

    //Go over all field names and save them and their types as tuples
    let fields_with_sqltypes = field_names.iter().map(|(ident, ty)| {
        let ident_quoted = ident.to_string();
        quote! {(#ident_quoted, <#ty as sqlite_traits::SqlTyped>::get_sql_type())}
    }).collect::<Vec<_>>();

    //Go over all field names and save them as well as a reference to their values as tuples
    let fields_with_values = field_names.iter().map(|(ident, _ty)|{
        let ident_quoted = ident.to_string(); //We need to convert name->&str
        quote! {(#ident_quoted, &self.#ident)}
    }).collect::<Vec<_>>();

    //Build the code to convert a sql row to the struct
    let mut from_row_string = format!("{}{{\n", table_name);
    from_row_string.push_str(&format!("\t{}:  row.get(\"{}\")?,\n", "id", "id")); //Extract ID
    for (ident, _ty) in field_names.iter(){ //Extract each field
        from_row_string.push_str(&format!("\t{}:  row.get(\"{}\")?,\n", ident.to_string(), ident.to_string()));
    }
    from_row_string.push('}');
    //Convert string to token stream
    let from_row_tokens = proc_macro2::TokenStream::from_str(&from_row_string).unwrap();

    //Create trait imple
    let expanded = quote! {
        impl sqlite_traits::SqlObject for #name{

            //Attach id field
            fn set_id(&mut self, id: Option<i64>){
                self.id = id;
            }

            fn get_id(&self) -> Option<i64>{
                return self.id;
            }

            //Return stored table name
            fn get_table_name() -> &'static str{
                return #table_name;
            }

            //We generated these above
            fn fields_with_sqltypes() -> Vec<(&'static str, sqlite_traits::SqlType)>{ 
                vec![#(#fields_with_sqltypes),*]
            }

            fn fields_with_values<'a>(&'a self) ->Vec<(&'static str, &'a dyn sqlite_traits::ToSql)>{
                vec![#(#fields_with_values),*]
            }

            fn deserialize_from_row(row: &sqlite_traits::Row) -> Result<Self, Box<dyn Error>> where Self: Sized{
                Ok(
                    #from_row_tokens
                )
            }
        }
    };

    // Hand the output tokens back to the compiler.
    expanded.into()
}

