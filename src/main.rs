use postgres::{Client, NoTls}; // To connect to Postgres Server , No TSL/SSL 
use postgres::Error as PostgresError;  //mport kiểu lỗi Error từ thư viện postgres, nhưng đổi tên lại thành PostgresError để dễ phân biệt với các lỗi khác.
use std::net::{TcpListener,TcpStream}; //Dùng để lắng nghe các kết nối TCP đến
use std::io::{Read,Write};             // Cho phép đọc dữ liệu từ TcpStream hoặc các nguồn khác (file, stdin,…).
//use std::env;                          //Dùng để lấy biến môi trường (environment variables) như DATABASE_URL, PORT, HOST, v.v.


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
const DB_URL: &str = "postgres://postgres:password@localhost:5432/postgres";
 //env!("DATABASE_URL");

//constants
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
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
    let listener = TcpListener::bind(format!("localhost:8080")).unwrap();
    println!("Server start at port 8080");

    //handle the client
    for stream in listener.incoming(){
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("Error {}", e);
            }
        }
    }
 }
 
 fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0;1024];
    let mut request = String::new();

    match stream.read(&mut buffer){
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let(status_line,content) = match &*request{
                r if r.starts_with("POST /users") => handle_post_request(r),
                r if r.starts_with("GET /users/") => handle_get_request(r),
                r if r.starts_with("GET /users") => handle_get_all_request(r),
              //  r if r.starts_with("PUT /users/") => handle_put_request(r),
                r if r.starts_with("DELETE /users/") => handle_delete_request(r),
                _ => (NOT_FOUND.to_string(),"404 Not Found".to_string()),
            };

          stream.write_all(format!("{}{}",status_line,content).as_bytes()).unwrap();
        }
        Err(e) =>{
            println!("Error: {}", e);
        }
    }
 }
 
 //CONTROLLERS
 fn handle_get_all_request(request: &str) -> (String , String) {
    //Making a connect to database
    //return client
    let mut client = match Client::connect(DB_URL, NoTls){
        Ok(client) => client,
        Err(_) => return (INTERNAL_SERVER_ERROR.to_string(), "Can not connect to server".to_string()),
    };

    let rows = match client.query("SELECT * FROM users", &[]) {
        Ok(rows) => rows,
        Err(_) => return (INTERNAL_SERVER_ERROR.to_string(),"Query error".to_string()),
    };

    //Khai bao mang rong
    let mut vec_users = Vec::new();
    for row in rows
    {
        let id:i32 = row.get(0);
        let name:String = row.get(1);
        let email:String = row.get(2);
        vec_users.push((id,name,email));
    }
    (OK_RESPONSE.to_string(),serde_json::to_string(&vec_users).unwrap())
 }

 //input : request 
 //output: string vector
 fn handle_post_request(request: &str) -> (String, String) {
    match (get_user_request_body(&request), Client::connect(DB_URL,NoTls)) {
        (Ok(user),Ok(mut client)) =>{
            client
                .execute("INSERT INTO users (name,email) VALUES ($1, $2)",
                         &[&user.name,&user.email]
            ).unwrap();

            (OK_RESPONSE.to_string(),"User created".to_string())
        } 
        _ => (INTERNAL_SERVER_ERROR.to_string(),"Error".to_string()),
    }
 }
/*
 fn handle_delete_request(r: &str) -> (String, String) {
    todo!()
 }
 
 fn handle_put_request(r: &str) -> (String, String) {
    todo!()
 }
 */ 
 
 fn handle_delete_request(request: &str)->(String,String){
    //Kiem tra ket noi server
    let mut client = match Client::connect(DB_URL, NoTls){
        Ok(client) => client,
        Err(_) => return (INTERNAL_SERVER_ERROR.to_string(),"Can not connect to server".to_string()),
    };

    let id = match get_id(&request).parse::<i32>() {
        Ok(id)=> id,
        _e => return (INTERNAL_SERVER_ERROR.to_string(),"Invalid ID".to_string()),
    };

    match client.execute("DELETE FROM users FROM id=$id", &[&id]){
        Ok(row_effected) => {
            if row_effected ==0 {
                (NOT_FOUND.to_string(),"User not found".to_string())
            }else{
                (OK_RESPONSE.to_string(),"User deleted successfully".to_string())
            }
        },
        Err(_) => return  (INTERNAL_SERVER_ERROR.to_string(),"Failed to delete user".to_string()),
    }
 }

//Get by Id
 fn handle_get_request(request: &str) -> (String, String) {
    //Kiem tra id coi hop le kg
    let id = match get_id(&request).parse::<i32>() {
        Ok(id) => id,
        _e => return (INTERNAL_SERVER_ERROR.to_string(),"Invalid ID".to_string()),
    };

    //Kiem tra ket noi server
    let mut client = match Client::connect(DB_URL, NoTls){
        Ok(client) => client,
        Err(_) => return (INTERNAL_SERVER_ERROR.to_string(),"Can not connect to server".to_string()),
    };

    match client.query_one("SELECT * FROM users WHERE id = $1", &[&id]) {
        Ok(row) =>{
            let user = User{
                id: row.get(0),
                name: row.get(1),
                email: row.get(2),
            };
            (
                OK_RESPONSE.to_string(),
                serde_json::to_string(&user).unwrap(),
            )
        }

        _ => (NOT_FOUND.to_string(),"No record returns".to_string()),
    }
 }

 
 //Create table
 fn set_database() -> Result<(),PostgresError>{

    //Connect to database
    let mut client = Client::connect(DB_URL, NoTls)?;

    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS users (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR NOT NULL,
                    email VARCHAR NOT NULL
                )"
    )?;
    Ok(()) // Last line is default to return. without ;
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
