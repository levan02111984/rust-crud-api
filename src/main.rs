use postgres::{Client,NoTls}; // To connect to Postgres Server , No TSL/SSL 
use postgres::Error as PostgresError;  //mport kiểu lỗi Error từ thư viện postgres, nhưng đổi tên lại thành PostgresError để dễ phân biệt với các lỗi khác.
use std::net::{TcpListener,TcpStream}; //Dùng để lắng nghe các kết nối TCP đến
use std::io::{Read,Write};             // Cho phép đọc dữ liệu từ TcpStream hoặc các nguồn khác (file, stdin,…).
use std::env;                          //Dùng để lấy biến môi trường (environment variables) như DATABASE_URL, PORT, HOST, v.v.


//Import các macro như #[derive(Serialize, Deserialize)] từ thư viện serde_derive.
//Giúp bạn dễ dàng chuyển đổi giữa struct ↔ JSON (hoặc các định dạng khác).
#[macro_use]
extern crate serde_derive;

//Model : User struct with id, name , email
#[derive(Serialize,Deserialize)]
struct  User{
    id: Option<i32>,
    name:String,
    email:String,
}

//DATABASE_URL
const DB_URL: &str = !env("DATABASE_URL");

//constants
const OK_RESPONSE: &str ="HTTP:/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str ="HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str ="HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

//main function
 fn main(){
    //set database
    // if let dung de xua ly Option hoac Result
    if let Err(e) = set_database() {
        println!("Error : {}",e);
        return;
    }


    //start server and print port
    let listener = TcpListener::bind(format!(0.0.0.0:8080)).unwrap();
    println!("Server start at port 8080");

    //handle the client
    for stream in listener.incoming(){
        match  stream {
            Ok(steam) => {
                handle_client(steam);
            }
            Err(e) => {
                println!("Error {}", e);
            }
        }
    }
 }
 

 //Create table
 fn set_database() -> Result<(),PostgresError>{

    //Connect to database
    let mut client = Client::connect(DB_URL, NoTls)?;

    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR NOT NULL,
                    email VARCHAR NOT NULL
                )",
         &[]
    )?
 }

 //get_id function
 //let request = "GET /api/user/1234 HTTP/1.1";
 fn get_id(request: &str) -> &str{
    request.split("/")
            .nth(2)
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or_default()
  }
   // Nhan request tu client va deserilize no thanh object
  //deserilize user from request body with the id
  fn get_user_request_body(request: &str) -> Result<User,serde_json::Error>{
     serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
  }
