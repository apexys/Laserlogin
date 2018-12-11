#![recursion_limit="1024"]
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, Data, Fields, Expr, Ident};
use quote::{ToTokens};
use std::str::FromStr;

const KNOWN_TYPES: [&'static str; 5] = ["i32", "i64", "f32", "f64", "String"];

fn type_name_to_sql_name(t: &str)-> &'static str{
    match t{
        "i32" => "INTEGER",
        "i64" => "INTEGER",
        "f32" => "REAL",
        "f64" => "REAL",
        "String" => "TEXT",
        _ => "TEXT"
    }
}

#[proc_macro_derive(SqlMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
     // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let table_name = &format!("{}", name);

    let mut init_str = String::new();
    let mut create_str = String::new();
    let mut update_str = String::new();
    let mut delete_str = String::new();

    let mut create_fields: Vec<Expr> = Vec::new();
    let mut update_fields;

    let ts: proc_macro2::TokenStream;
    let mut from_row_str = String::new();
    from_row_str.push_str(&format!{"{}",table_name});
    from_row_str.push_str("{\n");


    let fields_name = format!{"{}Fields", table_name};
    let fields_id: Ident = syn::parse_str(&fields_name).unwrap();
    let fs: proc_macro2::TokenStream;
    let fnames: proc_macro2::TokenStream;
    let mut fields_str = String::new();
    fields_str.push_str("pub struct ");
    fields_str.push_str(&fields_name);
    fields_str.push_str("{\n");
    fields_str.push_str("\tpub id: &'static str,\n");

    let mut fields_names = String::new();
    fields_names.push_str(&format!{"{}{{\n", &fields_name});
    fields_names.push_str("\tid: \"id\",\n");


    //let mut from_row_lines: Vec<Expr> = Vec::new();
    //from_row_lines.push(syn::parse_str("id: r.get_checked(0)?").unwrap());
    from_row_str.push_str("\tid: r.get_checked(0)?,\n");

    match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    init_str.push_str("CREATE TABLE IF NOT EXISTS ");
                    init_str.push_str(table_name);
                    init_str.push_str(" (");
                    init_str.push_str("id INTEGER PRIMARY KEY, ");

                    create_str.push_str("INSERT INTO ");
                    create_str.push_str(table_name);
                    create_str.push_str(" (");

                    update_str.push_str("UPDATE ");
                    update_str.push_str(table_name);
                    update_str.push_str(" SET ");

                    delete_str.push_str("DELETE FROM ");
                    delete_str.push_str(table_name);
                    delete_str.push_str(" WHERE id = ?");

                    let mut ctr = 0;
                    fields.named.iter().for_each(|f| {
                        let ident = &format!("{}", f.ident.clone().unwrap());
                        let typename = &format!{"{}", f.ty.clone().into_token_stream()};
                        if ident != "id"{
                            if ctr > 0{
                                init_str.push_str(", ");
                            }
                            init_str.push_str(ident);
                            init_str.push(' ');
                            init_str.push_str(type_name_to_sql_name(typename));

                            if ctr > 0{
                                create_str.push_str(", ");
                            }
                            create_str.push_str(ident);

                            if ctr > 0{
                                update_str.push_str(", ");
                            }
                            update_str.push_str(ident);
                            update_str.push_str(" = ?");

                            if ctr > 0{
                                fields_str.push_str(",\n");
                                fields_names.push_str(",\n");
                            }
                            fields_str.push_str(&format!{"\tpub {}: &'static str", &ident});
                            fields_names.push_str(&format!{"\t{}: \"{}\"", &ident, &ident});
                            


                            if ctr > 0{
                                from_row_str.push_str(",\n");
                            }

                            if KNOWN_TYPES.iter().any(|t| t == typename){ //If there's a direct correspondence to the type name
                                create_fields.push(syn::parse_str(&ident).unwrap());
                                //from_row_lines.push(syn::parse_str(&format!{"{}:  r.get_checked({})?", &ident, (ctr + 1)}).unwrap());
                                from_row_str.push_str(&format!{"\t{}:  r.get_checked({})?", &ident, (ctr + 1)});
                            }else{
                                let mut access_string = String::new();
                                //access_string.push("&format!{\"{}\",");
                                access_string.push_str(ident);
                                access_string.push_str(".to_str()");
                                create_fields.push(syn::parse_str(&access_string).unwrap());
                                //from_row_lines.push(syn::parse_str(&format!{"{}: {}::from_str(r.get_checked({}))?", &ident, &typename, (ctr + 1)}).unwrap());
                                from_row_str.push_str(&format!{"\t{}: {}::from_string(r.get_checked({})?)", &ident, &typename, (ctr + 1)});
                                //access_string.push("}");
                            }


                            ctr = ctr + 1;
                        }
                    });
                    init_str.push(')');
                    create_str.push(')');
                    create_str.push_str(" VALUES (");
                    for x in 0 .. ctr{
                        if x > 0 {
                            create_str.push_str(", ");
                        }
                        create_str.push('?');
                    }
                    create_str.push_str(")");

                    update_str.push_str(" WHERE id = ?");

                    update_fields = create_fields.clone();
                    update_fields.push(syn::parse_str("id").unwrap());

                    from_row_str.push_str("\n}");
                    ts = proc_macro2::TokenStream::from_str(&from_row_str).unwrap();

                    fields_str.push_str("\n}");
                    fields_names.push_str("\n}");

                    fs = proc_macro2::TokenStream::from_str(&fields_str).unwrap();
                    fnames = proc_macro2::TokenStream::from_str(&fields_names).unwrap();

                },
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
     };
    let expanded = quote! {
        #fs

        impl #name{
            pub fn fields() -> #fields_id{
                #fnames
            }
        }
        // The generated impl.
        impl sqlite_traits::dbobject::DbObject for #name{
            fn id(&self) -> i64{
                self.id
            }

            fn initialize(conn: &sqlite_traits::dbobject::DbConnection) ->Result<(), Box<std::error::Error>>{
                conn.execute(#init_str, std::iter::empty::<i64>())?;
                Ok(())
            }
            fn create(conn: &sqlite_traits::dbobject::DbConnection, o: &mut Self) ->Result<(), Box<std::error::Error>>{
                conn.execute(#create_str, &[#(&o.#create_fields as &sqlite_traits::ToSql),*])?;
                o.id = conn.last_insert_rowid();
                Ok(())
            }
            fn update(conn: &sqlite_traits::dbobject::DbConnection, o: &Self)->Result<(), Box<std::error::Error>>{
                conn.execute(#update_str, &[#(&o.#update_fields as &sqlite_traits::ToSql),*])?;
                Ok(())
            }
    
            fn delete(conn: &sqlite_traits::dbobject::DbConnection, o: &Self)->Result<(), Box<std::error::Error>>{
                conn.execute(#delete_str, &[&o.id])?;
                Ok(())
            }

            fn query(conn: sqlite_traits::dbobject::DbConnection) -> sqlite_traits::dbobject::Query<Self> where Self: sqlite_traits::dbobject::DbObject + Sized{
                sqlite_traits::dbobject::Query::new(conn, #table_name)
            }
            
            fn from_row(r: &sqlite_traits::dbobject::Row) -> Result<Self, Box<std::error::Error>>{
                Ok(#ts)
            }
        }
    };
    // Hand the output tokens back to the compiler.
    expanded.into()
}