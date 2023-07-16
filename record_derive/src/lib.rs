use proc_macro::TokenStream;
use quote::quote;
use syn;
use convert_case::{Case, Casing};
use syn::{Data, Fields};

#[proc_macro_derive(Record, attributes(primary, table_name))]
pub fn record_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_record(&ast)
}

fn impl_record(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let custom_table_name = ast.attrs.iter().find_map(|attr| {
        match &attr.meta {
            syn::Meta::NameValue(name_value) => {
                if name_value.path.segments.len() != 1 {
                    return None;
                }

                let name = &name_value.path.segments[0];

                match name.arguments {
                    syn::PathArguments::None => {
                        if name.ident.to_string().ne("table_name") {
                            return None;
                        }

                        let value = &name_value.value;
                        match value {
                            syn::Expr::Lit(e) => {
                                match &e.lit {
                                    syn::Lit::Str(st) => {
                                        Some(st.value())
                                    },
                                    _ => None
                                }
                            },
                            _ => None
                        }
                    }
                    _ => None
                }
            }
            _ => None
        }
    });

    let table_name = custom_table_name.unwrap_or(name.to_string().to_case(Case::UpperSnake));

    let get_all = format!("SELECT * FROM {}", table_name);

    let struct_data = match &ast.data {
        Data::Struct(d) => d,
        _ => panic!("fail")
    };

    let fields: Vec<_> = match &struct_data.fields {
        Fields::Named(fz) => fz,
        _ => panic!("fail")
    }.named.iter().collect();

    // println!("{:#?}", ast);

    let field_idents: Vec<_> = fields.iter().filter_map(|i| { i.ident.as_ref() }).collect();
    let prim_fields: Vec<_> = fields.iter().filter(|x| {
        x.attrs.iter().any(|it| {
            match &it.meta {
                syn::Meta::Path(p) => {
                    match p.leading_colon {
                        None => {},
                        _ => {
                            return false;
                        }
                    }

                    let segs: Vec<_> = p.segments.iter().collect();

                    if segs.len() != 1 {
                        return false;
                    }

                    let path_seg = segs[0];

                    match path_seg.arguments {
                        syn::PathArguments::None => path_seg.ident.to_string().eq("primary"),
                        _ => false
                    }
                },
                _ => {
                    false
                }
            }
        })
    }).copied().collect();

    if prim_fields.is_empty() {
        panic!("Missing primary field!");
    }

    let prim_field_idents: Vec<_> = prim_fields.iter().filter_map(|x| { x.ident.as_ref() }).collect();
    let prim_field_names: Vec<_> = prim_field_idents.iter().map(|x| { x.to_string() }).collect();

    let mut check_exists = format!("SELECT COUNT(*) FROM {} WHERE ", table_name);
    let check_exists_cond_vec: Vec<String> = prim_field_names.iter().map(|it| { format!("{} = ?", it) }).collect();
    let check_exists_cond = check_exists_cond_vec.join(" AND ");
    check_exists.push_str(&check_exists_cond);

    let mut insert_stmt = "INSERT INTO ".to_owned();
    insert_stmt.push_str(&table_name);
    insert_stmt.push_str(" (");
    let field_name_vec: Vec<String> = field_idents.iter().map(|field| { field.to_string() }).collect();
    let field_name_str = field_name_vec.join(", ");
    insert_stmt.push_str(&field_name_str);
    insert_stmt.push_str(") VALUES (");
    let placeholder_vec: Vec<&str> = field_idents.iter().map(|_| { "?" }).collect();
    let placeholder_str = placeholder_vec.join(", ");
    insert_stmt.push_str(&placeholder_str);
    insert_stmt.push_str(")");

    let non_prim_field_idents: Vec<_> = fields.iter().filter(|it| { !prim_fields.contains(it) }).filter_map(|it| { it.ident.as_ref() }).collect();

    let mut update_stmt = "UPDATE ".to_owned();
    update_stmt.push_str(&table_name);
    update_stmt.push_str(" SET ");
    let field_value_vec: Vec<String> = non_prim_field_idents.iter().map(|field| { format!("{} = ?", field.to_string()) }).collect();
    let field_value_str = field_value_vec.join(", ");
    update_stmt.push_str(&field_value_str);
    update_stmt.push_str(" WHERE ");
    update_stmt.push_str(&check_exists_cond);

    let mut truncate_stmt = "TRUNCATE ".to_string();
    truncate_stmt.push_str(&table_name);

    let update_field_idents = [non_prim_field_idents.as_slice(), prim_field_idents.as_slice()].concat();

    let gen = quote! {
        impl Record for #name {
            fn all<T: mysql::prelude::Queryable>(conn: &mut T) -> mysql::Result<Vec<#name>> {
                log::trace!(#get_all);
                conn.query_map(#get_all, |(#(#field_idents),*,)| {
                    #name { #(#field_idents),* }
                })
            }

            fn save<T: mysql::prelude::Queryable>(&mut self, conn: &mut T) -> mysql::Result<()> {
                let prim_field_vals = vec![#(self.#prim_field_idents.is_some()),*];
                let all_some = prim_field_vals.iter().all(|it| *{ it });

                if all_some {
                    log::trace!(#check_exists);
                    let count = conn.exec_fold(#check_exists, (#(&self.#prim_field_idents.as_ref().unwrap()),*,), 0, | last, row: (usize,) | {
                        last + row.0
                    }).unwrap();

                    if count == 0 {
                        log::trace!(#insert_stmt);
                        conn.exec_drop(#insert_stmt, (#(&self.#field_idents),*,))
                    } else {
                        log::trace!(#update_stmt);
                        conn.exec_drop(#update_stmt, (#(&self.#update_field_idents),*,))
                    }
                } else {
                    panic!();
                }
            }

            fn reload<T: mysql::prelude::Queryable>(&mut self, conn: &mut T) -> mysql::Result<()> {
                todo!()
            }

            fn destroy_all<T: mysql::prelude::Queryable>(conn: &mut T) -> mysql::Result<()> {
                log::trace!(#truncate_stmt);
                conn.exec_drop(#truncate_stmt, ())
            }

            fn save_all<T: mysql::prelude::Queryable>(conn: &mut T, items: &mut [#name]) -> mysql::Result<()> {
                conn.exec_batch(#insert_stmt, items.iter().map(|item| {
                    (#(&item.#field_idents),*,)
                }))
            }
        }
    };

    gen.into()
}