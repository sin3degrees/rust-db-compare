mod conf;

use std::io::{Write};
use conf::config::Config;
use mysql::*;
use mysql::prelude::Queryable;

fn conv_val_to_string(value: &Value) -> String {
    match value {
        Value::Bytes(bytes) => String::from_utf8_lossy(bytes).to_string(),
        Value::Int(i) => i.to_string(),
        Value::UInt(u) => u.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Double(d) => d.to_string(),
        Value::Date(year, month, day, hour, minute, second, micros) => format!(
            "{}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
            year, month, day, hour, minute, second, micros
        ),
        _ => "".to_string(),
    }
}

fn main() {
    let config = Config::new();
    let url_src = format!("mysql://{}:{}@{}:{}/{}",
        config.db_src.user,
        config.db_src.password,
        config.db_src.host,
        config.db_src.port,
        config.db_src.database,
    );
    let url_dst = format!("mysql://{}:{}@{}:{}/{}",
        config.db_dst.user,
        config.db_dst.password,
        config.db_dst.host,
        config.db_dst.port,
        config.db_dst.database,
    );
    let pool_src = Pool::new(&*url_src).unwrap();
    let pool_dst = Pool::new(&*url_dst).unwrap();
    let mut conn_src = pool_src.get_conn().unwrap();
    let mut conn_dst = pool_dst.get_conn().unwrap();

    let table_src: Vec<(String, String)> = conn_src.query("show full tables").unwrap();
    let table_dst: Vec<String> = conn_dst.query("show tables").unwrap();
    let tb_only = config.tb_only;
    let tb_ignore = config.tb_ignore;
    // 打开文件，如果文件不存在则创建，如果文件存在则清空文件以追加方式打开
    let mut file = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open("result.sql").unwrap();
    for x in table_src.iter() {
        let flag = (tb_only.len() == 0 || tb_only.contains(&x.0)) && !tb_ignore.contains(&x.0);
        if flag  {
            if table_dst.contains(&x.0) {
                if x.1 == "VIEW" {
                    let sql = format!("SELECT VIEW_DEFINITION FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_NAME = '{}'", x.0);
                    let mut view: Vec<String> = conn_src.query(sql).unwrap();
                    view[0] = view[0].replace(&*("`".to_owned() + &*config.db_src.database + "`."), "");
                    let sql = format!("ALTER VIEW {} AS {};\n\n", x.0, view[0]);
                    println!("{}", sql);
                    file.write_all(sql.as_bytes()).unwrap();
                } else {
                    let mut sql = format!("SHOW FULL COLUMNS FROM {}", x.0);
                    let col_map_src: Vec<(String, String, Option<Value>, String, String, Option<Value>, String, String, String)> = conn_src.query(sql).unwrap();
                    sql = format!("SHOW FULL COLUMNS FROM {}", x.0);
                    let col_map_dst: Vec<(String, String, Option<Value>, String, String, Option<Value>, String, String, String)> = conn_dst.query(sql).unwrap();
                    let new_cols = col_map_src.iter().filter(|x| !col_map_dst.iter().any(|y| y.0 == x.0)).collect::<Vec<_>>();
                    let mut col_sql = "".to_owned();
                    for x in new_cols.iter() {
                        let comment = " ".to_owned() + &*x.6.to_uppercase();
                        let default = if x.5.is_none() { "".to_owned() } else { "DEFAULT ".to_owned() + &*conv_val_to_string(x.5.as_ref().unwrap()) };
                        col_sql = col_sql + &*format!("\nADD COLUMN {} {} {}{}{} COMMENT '{}'", x.0, x.1, if x.3 == "YES" { "NULL" } else { "NOT NULL" }, default, if x.6 == "" { "" } else { &*comment }, x.8);
                    }
                    if col_sql != "" {
                        let sql = format!("ALTER TABLE {} {};\n\n", x.0, col_sql);
                        println!("{}", sql);
                        file.write_all(sql.as_bytes()).unwrap();
                    }
                }
            } else {
                if x.1 == "VIEW" {
                    let sql = format!("SELECT VIEW_DEFINITION FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_NAME = '{}'", x.0);
                    let mut view: Vec<String> = conn_src.query(sql).unwrap();
                    view[0] = view[0].replace(&*("`".to_owned() + &*config.db_src.database + "`."), "");
                    let sql = format!("CREATE VIEW {} AS {};\n\n", x.0, view[0]);
                    println!("{}", sql);
                    file.write_all(sql.as_bytes()).unwrap();
                } else {
                    let sql = format!("SHOW FULL COLUMNS FROM {}", x.0);
                    //field, f_type, collation, null, key, default, extra, privileges, comment
                    let col_map_src: Vec<(String, String, Option<Value>, String, String, Option<Value>, String, String, String)> = conn_src.query(sql).unwrap();
                    let mut col_sql = "".to_owned();
                    let mut pri_str = "".to_owned();
                    for x in col_map_src.iter() {
                        let comment = " ".to_owned() + &*x.6.to_uppercase();
                        let default = if x.5.is_none() { "".to_owned() } else { "DEFAULT ".to_owned() + &*conv_val_to_string(x.5.as_ref().unwrap()) };
                        col_sql = col_sql.to_owned() + &*format!("\n{} {} {}{}{} COMMENT '{}'", x.0, x.1, if x.3 == "YES" { "NULL" } else { "NOT NULL" }, default, if x.6 == "" { "" } else { &*comment }, x.8);
                        if x.4 == "PRI" {
                            pri_str = format!("\nPRIMARY KEY ({}) USING BTREE", x.0);
                        }
                    }
                    let sql = format!("CREATE TABLE {} ({}, {});\n\n", x.0, col_sql, pri_str);
                    println!("{}", sql);
                    file.write_all(sql.as_bytes()).unwrap();
                }
            }
        }
    }
}
